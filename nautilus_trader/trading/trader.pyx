# -------------------------------------------------------------------------------------------------
#  Copyright (C) 2015-2022 Nautech Systems Pty Ltd. All rights reserved.
#  https://nautechsystems.io
#
#  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
#  You may not use this file except in compliance with the License.
#  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
#
#  Unless required by applicable law or agreed to in writing, software
#  distributed under the License is distributed on an "AS IS" BASIS,
#  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#  See the License for the specific language governing permissions and
#  limitations under the License.
# -------------------------------------------------------------------------------------------------

"""
The `Trader` class is intended to manage a portfolio of trading strategies within
a running instance of the platform.

A running instance could be either a test/backtest or live implementation - the
`Trader` will operate in the same way.
"""

from typing import Any, Callable

import pandas as pd

from nautilus_trader.analysis import statistics
from nautilus_trader.analysis.analyzer import PortfolioAnalyzer
from nautilus_trader.analysis.reporter import ReportProvider

from nautilus_trader.accounting.accounts.base cimport Account
from nautilus_trader.common.actor cimport Actor
from nautilus_trader.common.clock cimport Clock
from nautilus_trader.common.component cimport Component
from nautilus_trader.common.logging cimport Logger
from nautilus_trader.core.correctness cimport Condition
from nautilus_trader.data.engine cimport DataEngine
from nautilus_trader.execution.engine cimport ExecutionEngine
from nautilus_trader.model.identifiers cimport TraderId
from nautilus_trader.model.identifiers cimport Venue
from nautilus_trader.msgbus.bus cimport MessageBus
from nautilus_trader.risk.engine cimport RiskEngine
from nautilus_trader.trading.strategy cimport TradingStrategy


cdef class Trader(Component):
    """
    Provides a trader for managing a portfolio of trading strategies.

    Parameters
    ----------
    trader_id : TraderId
        The ID for the trader.
    msgbus : MessageBus
        The message bus for the trader.
    cache : Cache
        The cache for the trader.
    portfolio : Portfolio
        The portfolio for the trader.
    data_engine : DataEngine
        The data engine for the trader.
    risk_engine : RiskEngine
        The risk engine for the trader.
    exec_engine : ExecutionEngine
        The execution engine for the trader.
    clock : Clock
        The clock for the trader.
    logger : Logger
        The logger for the trader.
    config : dict[str, Any]
        The configuration for the trader.

    Raises
    ------
    ValueError
        If `portfolio` is not equal to the `exec_engine` portfolio.
    ValueError
        If `strategies` is ``None``.
    ValueError
        If `strategies` is empty.
    TypeError
        If `strategies` contains a type other than `TradingStrategy`.
    """

    def __init__(
        self,
        TraderId trader_id not None,
        MessageBus msgbus not None,
        Cache cache not None,
        Portfolio portfolio not None,
        DataEngine data_engine not None,
        RiskEngine risk_engine not None,
        ExecutionEngine exec_engine not None,
        Clock clock not None,
        Logger logger not None,
        dict config=None,
    ):
        if config is None:
            config = {}
        super().__init__(
            clock=clock,
            logger=logger,
            component_id=trader_id,
            msgbus=msgbus,
            config=config,
        )

        self._cache = cache
        self._portfolio = portfolio
        self._data_engine = data_engine
        self._risk_engine = risk_engine
        self._exec_engine = exec_engine

        self._actors = []
        self._strategies = []

        self.analyzer = PortfolioAnalyzer()

        # Register default statistics
        self.analyzer.register_statistic(statistics.winner_max.MaxWinner())
        self.analyzer.register_statistic(statistics.winner_avg.AvgWinner())
        self.analyzer.register_statistic(statistics.winner_min.MinWinner())
        self.analyzer.register_statistic(statistics.loser_min.MinLoser())
        self.analyzer.register_statistic(statistics.loser_avg.AvgLoser())
        self.analyzer.register_statistic(statistics.loser_max.MaxLoser())
        self.analyzer.register_statistic(statistics.expectancy.Expectancy())
        self.analyzer.register_statistic(statistics.win_rate.WinRate())
        self.analyzer.register_statistic(statistics.returns_volatility.ReturnsVolatility())
        self.analyzer.register_statistic(statistics.returns_avg.ReturnsAverage())
        self.analyzer.register_statistic(statistics.returns_avg_loss.ReturnsAverageLoss())
        self.analyzer.register_statistic(statistics.returns_avg_win.ReturnsAverageWin())
        self.analyzer.register_statistic(statistics.sharpe_ratio.SharpeRatio())
        self.analyzer.register_statistic(statistics.sortino_ratio.SortinoRatio())
        self.analyzer.register_statistic(statistics.profit_factor.ProfitFactor())
        self.analyzer.register_statistic(statistics.risk_return_ratio.RiskReturnRatio())
        self.analyzer.register_statistic(statistics.long_ratio.LongRatio())

    cdef list actors_c(self):
        return self._actors

    cdef list strategies_c(self):
        return self._strategies

    cpdef list actor_ids(self):
        """
        Return the actor IDs loaded in the trader.

        Returns
        -------
        list[ComponentId]

        """
        return sorted([actor.id for actor in self._actors])

    cpdef list strategy_ids(self):
        """
        Return the strategy IDs loaded in the trader.

        Returns
        -------
        list[StrategyId]

        """
        return sorted([strategy.id for strategy in self._strategies])

    cpdef dict actor_states(self):
        """
        Return the traders actor states.

        Returns
        -------
        dict[ComponentId, str]

        """
        cdef Actor a
        return {a.id: a.state_string_c() for a in self._actors}

    cpdef dict strategy_states(self):
        """
        Return the traders strategy states.

        Returns
        -------
        dict[StrategyId, str]

        """
        cdef TradingStrategy s
        return {s.id: s.state_string_c() for s in self._strategies}

# -- ACTION IMPLEMENTATIONS ------------------------------------------------------------------------

    cpdef void _start(self) except *:
        if not self._strategies:
            self._log.error(f"No strategies loaded.")
            return

        cdef Actor actor
        for actor in self._actors:
            actor.start()

        cdef TradingStrategy strategy
        for strategy in self._strategies:
            strategy.start()

    cpdef void _stop(self) except *:
        cdef Actor actor
        for actor in self._actors:
            if actor.is_running_c():
                actor.stop()
            else:
                self._log.warning(f"{actor} already stopped.")

        cdef TradingStrategy strategy
        for strategy in self._strategies:
            if strategy.is_running_c():
                strategy.stop()
            else:
                self._log.warning(f"{strategy} already stopped.")

    cpdef void _reset(self) except *:
        cdef Actor actor
        for actor in self._actors:
            actor.reset()

        cdef TradingStrategy strategy
        for strategy in self._strategies:
            strategy.reset()

        self._portfolio.reset()
        self.analyzer.reset()

    cpdef void _dispose(self) except *:
        cdef Actor actor
        for actor in self._actors:
            actor.dispose()

        cdef TradingStrategy strategy
        for strategy in self._strategies:
            strategy.dispose()

# --------------------------------------------------------------------------------------------------

    cpdef void add_strategy(self, TradingStrategy strategy) except *:
        """
        Add the given trading strategy to the trader.

        Parameters
        ----------
        strategy : TradingStrategy
            The trading strategy to add and register.

        Raises
        ------
        KeyError
            If `strategy.id` already exists in the trader.
        ValueError
            If `strategy.state` is ``RUNNING`` or ``DISPOSED``.

        """
        Condition.not_none(strategy, "strategy")
        Condition.not_in(strategy, self._strategies, "strategy", "_strategies")
        Condition.true(not strategy.is_running_c(), "strategy.state was RUNNING")
        Condition.true(not strategy.is_disposed_c(), "strategy.state was DISPOSED")

        if self.is_running_c():
            self._log.error("Cannot add a strategy to a running trader.")
            return

        # Wire strategy into trader
        strategy.register(
            trader_id=self.id,
            portfolio=self._portfolio,
            msgbus=self._msgbus,
            cache=self._cache,
            clock=self._clock.__class__(),  # Clock per strategy
            logger=self._log.get_logger(),
        )

        self._exec_engine.register_oms_type(strategy)
        self._strategies.append(strategy)

        self._log.info(f"Registered TradingStrategy {strategy}.")

    cpdef void add_strategies(self, list strategies: [TradingStrategy]) except *:
        """
        Add the given trading strategies to the trader.

        Parameters
        ----------
        strategies : list[TradingStrategies]
            The trading strategies to add and register.

        Raises
        ------
        ValueError
            If `strategies` is ``None`` or empty.

        """
        Condition.not_empty(strategies, "strategies")

        cdef TradingStrategy strategy
        for strategy in strategies:
            self.add_strategy(strategy)

    cpdef void add_actor(self, Actor actor) except *:
        """
        Add the given custom component to the trader.

        Parameters
        ----------
        actor : Actor
            The actor to add and register.

        Raises
        ------
        KeyError
            If `component.id` already exists in the trader.
        ValueError
            If `component.state` is ``RUNNING`` or ``DISPOSED``.

        """
        Condition.not_in(actor, self._actors, "actor", "_actors")
        Condition.true(not actor.is_running_c(), "actor.state was RUNNING")
        Condition.true(not actor.is_disposed_c(), "actor.state was DISPOSED")

        if self.is_running_c():
            self._log.error("Cannot add component to a running trader.")
            return

        # Wire component into trader
        actor.register_base(
            trader_id=self.id,
            msgbus=self._msgbus,
            cache=self._cache,
            clock=self._clock.__class__(),  # Clock per component
            logger=self._log.get_logger(),
        )

        self._actors.append(actor)

        self._log.info(f"Registered Component {actor}.")

    cpdef void add_actors(self, list actors: [Actor]) except *:
        """
        Add the given actors to the trader.

        Parameters
        ----------
        actors : list[TradingStrategies]
            The actors to add and register.

        Raises
        ------
        ValueError
            If `actors` is ``None`` or empty.

        """
        Condition.not_empty(actors, "actors")

        cdef Actor actor
        for actor in actors:
            self.add_actor(actor)

    cpdef void clear_strategies(self) except *:
        """
        Dispose and clear all strategies held by the trader.

        Raises
        ------
        ValueError
            If state is ``RUNNING``.

        """
        if self.is_running_c():
            self._log.error("Cannot clear the strategies of a running trader.")
            return

        for strategy in self._strategies:
            strategy.dispose()

        self._strategies.clear()

    cpdef void clear_actors(self) except *:
        """
        Dispose and clear all actors held by the trader.

        Raises
        ------
        ValueError
            If state is ``RUNNING``.

        """
        if self.is_running_c():
            self._log.error("Cannot clear the actors of a running trader.")
            return

        for actor in self._actors:
            actor.dispose()

        self._actors.clear()

    cpdef void subscribe(self, str topic, handler: Callable[[Any], None]) except *:
        """
        Subscribe to the given message topic with the given callback handler.

        Parameters
        ----------
        topic : str
            The topic for the subscription. May include wildcard glob patterns.
        handler : Callable[[Any], None]
            The handler for the subscription.

        """
        self._msgbus.subscribe(topic=topic, handler=handler)

    cpdef void unsubscribe(self, str topic, handler: Callable[[Any], None]) except *:
        """
        Unsubscribe the given handler from the given message topic.

        Parameters
        ----------
        topic : str, optional
            The topic to unsubscribe from. May include wildcard glob patterns.
        handler : Callable[[Any], None]
            The handler for the subscription.

        """
        self._msgbus.unsubscribe(topic=topic, handler=handler)

    cpdef void save(self) except *:
        """
        Save all strategy states to the execution cache.
        """
        for strategy in self._strategies:
            self._cache.update_strategy(strategy)

    cpdef void load(self) except *:
        """
        Load all strategy states from the execution cache.
        """
        for strategy in self._strategies:
            self._cache.load_strategy(strategy)

    cpdef void check_residuals(self) except *:
        """
        Check for residual open state such as open orders or open positions.
        """
        self._exec_engine.check_residuals()

    cpdef object generate_orders_report(self):
        """
        Generate an orders report.

        Returns
        -------
        pd.DataFrame

        """
        return ReportProvider.generate_orders_report(self._cache.orders())

    cpdef object generate_order_fills_report(self):
        """
        Generate an order fills report.

        Returns
        -------
        pd.DataFrame

        """
        return ReportProvider.generate_order_fills_report(self._cache.orders())

    cpdef object generate_positions_report(self):
        """
        Generate a positions report.

        Returns
        -------
        pd.DataFrame

        """
        return ReportProvider.generate_positions_report(self._cache.positions())

    cpdef object generate_account_report(self, Venue venue):
        """
        Generate an account report.

        Returns
        -------
        pd.DataFrame

        """
        cdef Account account = self._cache.account_for_venue(venue)
        if account is None:
            return pd.DataFrame()
        return ReportProvider.generate_account_report(account)
