# -------------------------------------------------------------------------------------------------
#  Copyright (C) 2015-2021 Nautech Systems Pty Ltd. All rights reserved.
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

from decimal import Decimal

from libc.stdint cimport int64_t
from libc.stdint cimport uint8_t

from nautilus_trader.model.c_enums.order_side cimport OrderSide
from nautilus_trader.model.c_enums.position_side cimport PositionSide
from nautilus_trader.model.currency cimport Currency
from nautilus_trader.model.events.order cimport OrderFilled
from nautilus_trader.model.identifiers cimport AccountId
from nautilus_trader.model.identifiers cimport ClientOrderId
from nautilus_trader.model.identifiers cimport ExecutionId
from nautilus_trader.model.identifiers cimport InstrumentId
from nautilus_trader.model.identifiers cimport PositionId
from nautilus_trader.model.identifiers cimport StrategyId
from nautilus_trader.model.identifiers cimport TraderId
from nautilus_trader.model.objects cimport Money
from nautilus_trader.model.objects cimport Price
from nautilus_trader.model.objects cimport Quantity


cdef class Position:
    cdef list _events
    cdef list _execution_ids
    cdef object _buy_qty
    cdef object _sell_qty
    cdef dict _commissions

    cdef readonly TraderId trader_id
    """The trader ID associated with the position.\n\n:returns: `TraderId`"""
    cdef readonly StrategyId strategy_id
    """The strategy ID associated with the position.\n\n:returns: `StrategyId`"""
    cdef readonly InstrumentId instrument_id
    """The position instrument ID.\n\n:returns: `InstrumentId`"""
    cdef readonly PositionId id
    """The position ID.\n\n:returns: `PositionId`"""
    cdef readonly AccountId account_id
    """The account ID associated with the position.\n\n:returns: `AccountId`"""
    cdef readonly ClientOrderId from_order
    """The client order ID for the order which initially opened the position.\n\n:returns: `ClientOrderId`"""
    cdef readonly OrderSide entry
    """The position entry order side.\n\n:returns: `OrderSide`"""
    cdef readonly PositionSide side
    """The current position side.\n\n:returns: `PositionSide`"""
    cdef readonly object net_qty
    """The current net quantity (positive for LONG, negative for SHORT).\n\n:returns: `Decimal`"""
    cdef readonly Quantity quantity
    """The current open quantity.\n\n:returns: `Quantity`"""
    cdef readonly Quantity peak_qty
    """The peak directional quantity reached by the position.\n\n:returns: `Quantity`"""
    cdef readonly uint8_t price_precision
    """The price precision for the position.\n\n:returns: `uint8`"""
    cdef readonly uint8_t size_precision
    """The size precision for the position.\n\n:returns: `uint8`"""
    cdef readonly Quantity multiplier
    """The multiplier for the positions instrument.\n\n:returns: `Quantity`"""
    cdef readonly bint is_inverse
    """If the quantity is expressed in quote currency.\n\n:returns: `bool`"""
    cdef readonly Currency quote_currency
    """The position quote currency.\n\n:returns: `Currency`"""
    cdef readonly Currency base_currency
    """The position base currency (if applicable).\n\n:returns: `Currency` or None"""
    cdef readonly Currency cost_currency
    """The position cost currency (for PnL).\n\n:returns: `Currency`"""
    cdef readonly int64_t ts_init
    """The UNIX timestamp (nanoseconds) when the position was initialized.\n\n:returns: `int64`"""
    cdef readonly int64_t ts_opened
    """The UNIX timestamp (nanoseconds) when the position was opened.\n\n:returns: `int64`"""
    cdef readonly int64_t ts_last
    """The UNIX timestamp (nanoseconds) when the last fill occurred.\n\n:returns: `int64`"""
    cdef readonly int64_t ts_closed
    """The UNIX timestamp (nanoseconds) when the position was closed.\n\n:returns: `int64`"""
    cdef readonly int64_t duration_ns
    """The total open duration (nanoseconds).\n\n:returns: `int64`"""
    cdef readonly object avg_px_open
    """The average open price.\n\n:returns: `Decimal`"""
    cdef readonly object avg_px_close
    """The average closing price.\n\n:returns: `Decimal` or `None`"""
    cdef readonly object realized_points
    """The current realized points for the position.\n\n:returns: `Decimal`"""
    cdef readonly object realized_return
    """The current realized return for the position.\n\n:returns: `Decimal`"""
    cdef readonly Money realized_pnl
    """The current realized PnL for the position (including commissions).\n\n:returns: `Money`"""

    cpdef str info(self)
    cpdef dict to_dict(self)

    cdef list client_order_ids_c(self)
    cdef list venue_order_ids_c(self)
    cdef list execution_ids_c(self)
    cdef list events_c(self)
    cdef OrderFilled last_event_c(self)
    cdef ExecutionId last_execution_id_c(self)
    cdef int event_count_c(self) except *
    cdef bint is_long_c(self) except *
    cdef bint is_short_c(self) except *
    cdef bint is_open_c(self) except *
    cdef bint is_closed_c(self) except *

    @staticmethod
    cdef PositionSide side_from_order_side_c(OrderSide side) except *
    cpdef bint is_opposite_side(self, OrderSide side) except *

    cpdef void apply(self, OrderFilled fill) except *

    cpdef Money notional_value(self, Price last)
    cpdef Money calculate_pnl(self, avg_px_open: Decimal, avg_px_close: Decimal, quantity: Decimal)
    cpdef Money unrealized_pnl(self, Price last)
    cpdef Money total_pnl(self, Price last)
    cpdef list commissions(self)

    cdef void _handle_buy_order_fill(self, OrderFilled fill) except *
    cdef void _handle_sell_order_fill(self, OrderFilled fill) except *
    cdef object _calculate_avg_px(self, avg_px: Decimal, qty: Decimal, OrderFilled fill)
    cdef object _calculate_avg_px_open_px(self, OrderFilled fill)
    cdef object _calculate_avg_px_close_px(self, OrderFilled fill)
    cdef object _calculate_points(self, avg_px_open: Decimal, avg_px_close: Decimal)
    cdef object _calculate_points_inverse(self, avg_px_open: Decimal, avg_px_close: Decimal)
    cdef object _calculate_return(self, avg_px_open: Decimal, avg_px_close: Decimal)
    cdef object _calculate_pnl(self, avg_px_open: Decimal, avg_px_close: Decimal, quantity: Decimal)
