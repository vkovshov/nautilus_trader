// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2023 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    fmt::{Debug, Display, Formatter},
    hash::{Hash, Hasher},
    str::FromStr,
};

use nautilus_core::{python::to_pyvalue_err, serialization::Serializable, time::UnixNanos};
use pyo3::{prelude::*, pyclass::CompareOp, types::PyDict};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror;

use crate::{
    enums::{AggregationSource, BarAggregation, PriceType},
    identifiers::instrument_id::InstrumentId,
    types::{price::Price, quantity::Quantity},
};

/// Represents a bar aggregation specification including a step, aggregation
/// method/rule and price type.
#[repr(C)]
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[pyclass]
pub struct BarSpecification {
    /// The step for binning samples for bar aggregation.
    pub step: usize,
    /// The type of bar aggregation.
    pub aggregation: BarAggregation,
    /// The price type to use for aggregation.
    pub price_type: PriceType,
}

impl Display for BarSpecification {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}-{}", self.step, self.aggregation, self.price_type)
    }
}

/// Represents a bar type including the instrument ID, bar specification and
/// aggregation source.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[pyclass]
pub struct BarType {
    /// The bar types instrument ID.
    pub instrument_id: InstrumentId,
    /// The bar types specification.
    pub spec: BarSpecification,
    /// The bar types aggregation source.
    pub aggregation_source: AggregationSource,
}

#[derive(thiserror::Error, Debug)]
#[error("Error parsing `BarType` from '{input}', invalid token: '{token}' at position {position}")]
pub struct BarTypeParseError {
    input: String,
    token: String,
    position: usize,
}

impl FromStr for BarType {
    type Err = BarTypeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: Requires handling some trait related thing
        #[allow(clippy::needless_collect)]
        let pieces: Vec<&str> = s.rsplitn(5, '-').collect();
        let rev_pieces: Vec<&str> = pieces.into_iter().rev().collect();
        if rev_pieces.len() != 5 {
            return Err(BarTypeParseError {
                input: s.to_string(),
                token: "".to_string(),
                position: 0,
            });
        }

        let instrument_id =
            InstrumentId::from_str(rev_pieces[0]).map_err(|_| BarTypeParseError {
                input: s.to_string(),
                token: rev_pieces[0].to_string(),
                position: 0,
            })?;

        let step = rev_pieces[1].parse().map_err(|_| BarTypeParseError {
            input: s.to_string(),
            token: rev_pieces[1].to_string(),
            position: 1,
        })?;
        let aggregation =
            BarAggregation::from_str(rev_pieces[2]).map_err(|_| BarTypeParseError {
                input: s.to_string(),
                token: rev_pieces[2].to_string(),
                position: 2,
            })?;
        let price_type = PriceType::from_str(rev_pieces[3]).map_err(|_| BarTypeParseError {
            input: s.to_string(),
            token: rev_pieces[3].to_string(),
            position: 3,
        })?;
        let aggregation_source =
            AggregationSource::from_str(rev_pieces[4]).map_err(|_| BarTypeParseError {
                input: s.to_string(),
                token: rev_pieces[4].to_string(),
                position: 4,
            })?;

        Ok(BarType {
            instrument_id,
            spec: BarSpecification {
                step,
                aggregation,
                price_type,
            },
            aggregation_source,
        })
    }
}

impl Display for BarType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}-{}-{}",
            self.instrument_id, self.spec, self.aggregation_source
        )
    }
}

impl Serialize for BarType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for BarType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        BarType::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl BarType {
    fn __richcmp__(&self, other: &Self, op: CompareOp, py: Python<'_>) -> Py<PyAny> {
        match op {
            CompareOp::Eq => self.eq(other).into_py(py),
            CompareOp::Ne => self.ne(other).into_py(py),
            _ => py.NotImplemented(),
        }
    }

    fn __hash__(&self) -> isize {
        let mut h = DefaultHasher::new();
        self.hash(&mut h);
        h.finish() as isize
    }

    fn __str__(&self) -> String {
        self.to_string()
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }
}

/// Represents an aggregated bar.
#[repr(C)]
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[pyclass]
pub struct Bar {
    /// The bar type for this bar.
    pub bar_type: BarType,
    /// The bars open price.
    pub open: Price,
    /// The bars high price.
    pub high: Price,
    /// The bars low price.
    pub low: Price,
    /// The bars close price.
    pub close: Price,
    /// The bars volume.
    pub volume: Quantity,
    /// The UNIX timestamp (nanoseconds) when the data event occurred.
    pub ts_event: UnixNanos,
    /// The UNIX timestamp (nanoseconds) when the data object was initialized.
    pub ts_init: UnixNanos,
}

impl Bar {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bar_type: BarType,
        open: Price,
        high: Price,
        low: Price,
        close: Price,
        volume: Quantity,
        ts_event: UnixNanos,
        ts_init: UnixNanos,
    ) -> Self {
        Self {
            bar_type,
            open,
            high,
            low,
            close,
            volume,
            ts_event,
            ts_init,
        }
    }

    /// Returns the metadata for the type, for use with serialization formats.
    pub fn get_metadata(
        bar_type: &BarType,
        price_precision: u8,
        size_precision: u8,
    ) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        let instrument_id = bar_type.instrument_id;
        metadata.insert("bar_type".to_string(), bar_type.to_string());
        metadata.insert("instrument_id".to_string(), instrument_id.to_string());
        metadata.insert("price_precision".to_string(), price_precision.to_string());
        metadata.insert("size_precision".to_string(), size_precision.to_string());
        metadata
    }

    /// Create a new [`Bar`] extracted from the given [`PyAny`].
    pub fn from_pyobject(obj: &PyAny) -> PyResult<Self> {
        let bar_type_obj: &PyAny = obj.getattr("bar_type")?.extract()?;
        let bar_type_str = bar_type_obj.call_method0("__str__")?.extract()?;
        let bar_type = BarType::from_str(bar_type_str)
            .map_err(to_pyvalue_err)
            .unwrap();

        let open_py: &PyAny = obj.getattr("open")?;
        let price_prec: u8 = open_py.getattr("precision")?.extract()?;
        let open_raw: i64 = open_py.getattr("raw")?.extract()?;
        let open = Price::from_raw(open_raw, price_prec);

        let high_py: &PyAny = obj.getattr("high")?;
        let high_raw: i64 = high_py.getattr("raw")?.extract()?;
        let high = Price::from_raw(high_raw, price_prec);

        let low_py: &PyAny = obj.getattr("low")?;
        let low_raw: i64 = low_py.getattr("raw")?.extract()?;
        let low = Price::from_raw(low_raw, price_prec);

        let close_py: &PyAny = obj.getattr("close")?;
        let close_raw: i64 = close_py.getattr("raw")?.extract()?;
        let close = Price::from_raw(close_raw, price_prec);

        let volume_py: &PyAny = obj.getattr("volume")?;
        let volume_raw: u64 = volume_py.getattr("raw")?.extract()?;
        let volume_prec: u8 = volume_py.getattr("precision")?.extract()?;
        let volume = Quantity::from_raw(volume_raw, volume_prec);

        let ts_event: UnixNanos = obj.getattr("ts_event")?.extract()?;
        let ts_init: UnixNanos = obj.getattr("ts_init")?.extract()?;

        Ok(Self::new(
            bar_type, open, high, low, close, volume, ts_event, ts_init,
        ))
    }
}

impl Serializable for Bar {}

impl Display for Bar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{},{},{},{},{},{},{}",
            self.bar_type, self.open, self.high, self.low, self.close, self.volume, self.ts_event
        )
    }
}

#[cfg(feature = "python")]
#[pymethods]
#[allow(clippy::too_many_arguments)]
impl Bar {
    #[new]
    fn py_new(
        bar_type: BarType,
        open: Price,
        high: Price,
        low: Price,
        close: Price,
        volume: Quantity,
        ts_event: UnixNanos,
        ts_init: UnixNanos,
    ) -> Self {
        Self::new(bar_type, open, high, low, close, volume, ts_event, ts_init)
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp, py: Python<'_>) -> Py<PyAny> {
        match op {
            CompareOp::Eq => self.eq(other).into_py(py),
            CompareOp::Ne => self.ne(other).into_py(py),
            _ => py.NotImplemented(),
        }
    }

    fn __hash__(&self) -> isize {
        let mut h = DefaultHasher::new();
        self.hash(&mut h);
        h.finish() as isize
    }

    fn __str__(&self) -> String {
        self.to_string()
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }

    #[getter]
    fn bar_type(&self) -> BarType {
        self.bar_type
    }

    #[getter]
    fn open(&self) -> Price {
        self.open
    }

    #[getter]
    fn high(&self) -> Price {
        self.high
    }

    #[getter]
    fn low(&self) -> Price {
        self.low
    }

    #[getter]
    fn close(&self) -> Price {
        self.close
    }

    #[getter]
    fn volume(&self) -> Quantity {
        self.volume
    }

    #[getter]
    fn ts_event(&self) -> UnixNanos {
        self.ts_event
    }

    #[getter]
    fn ts_init(&self) -> UnixNanos {
        self.ts_init
    }

    /// Return a dictionary representation of the object.
    pub fn as_dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        // Serialize object to JSON bytes
        let json_str = serde_json::to_string(self).map_err(to_pyvalue_err)?;
        // Parse JSON into a Python dictionary
        let py_dict: Py<PyDict> = PyModule::import(py, "json")?
            .call_method("loads", (json_str,), None)?
            .extract()?;
        Ok(py_dict)
    }

    /// Return a new object from the given dictionary representation.
    #[staticmethod]
    pub fn from_dict(py: Python<'_>, values: Py<PyDict>) -> PyResult<Self> {
        // Extract to JSON string
        let json_str: String = PyModule::import(py, "json")?
            .call_method("dumps", (values,), None)?
            .extract()?;

        // Deserialize to object
        let instance = serde_json::from_slice(&json_str.into_bytes()).map_err(to_pyvalue_err)?;
        Ok(instance)
    }

    #[staticmethod]
    fn from_json(data: Vec<u8>) -> PyResult<Self> {
        Self::from_json_bytes(data).map_err(to_pyvalue_err)
    }

    #[staticmethod]
    fn from_msgpack(data: Vec<u8>) -> PyResult<Self> {
        Self::from_msgpack_bytes(data).map_err(to_pyvalue_err)
    }

    /// Return JSON encoded bytes representation of the object.
    fn as_json(&self, py: Python<'_>) -> Py<PyAny> {
        // Unwrapping is safe when serializing a valid object
        self.as_json_bytes().unwrap().into_py(py)
    }

    /// Return MsgPack encoded bytes representation of the object.
    fn as_msgpack(&self, py: Python<'_>) -> Py<PyAny> {
        // Unwrapping is safe when serializing a valid object
        self.as_msgpack_bytes().unwrap().into_py(py)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Tests
////////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::{
        enums::BarAggregation,
        identifiers::{symbol::Symbol, venue::Venue},
    };

    fn create_stub_bar() -> Bar {
        let instrument_id = InstrumentId {
            symbol: Symbol::new("AUDUSD").unwrap(),
            venue: Venue::new("SIM").unwrap(),
        };
        let bar_spec = BarSpecification {
            step: 1,
            aggregation: BarAggregation::Minute,
            price_type: PriceType::Bid,
        };
        let bar_type = BarType {
            instrument_id,
            spec: bar_spec,
            aggregation_source: AggregationSource::External,
        };
        Bar {
            bar_type: bar_type.clone(),
            open: Price::from("1.00001"),
            high: Price::from("1.00004"),
            low: Price::from("1.00002"),
            close: Price::from("1.00003"),
            volume: Quantity::from("100000"),
            ts_event: 0,
            ts_init: 1,
        }
    }

    #[rstest]
    fn test_bar_spec_string_reprs() {
        let bar_spec = BarSpecification {
            step: 1,
            aggregation: BarAggregation::Minute,
            price_type: PriceType::Bid,
        };
        assert_eq!(bar_spec.to_string(), "1-MINUTE-BID");
        assert_eq!(format!("{bar_spec}"), "1-MINUTE-BID");
    }

    #[rstest]
    fn test_bar_type_parse_valid() {
        let input = "BTCUSDT-PERP.BINANCE-1-MINUTE-LAST-EXTERNAL";
        let bar_type = BarType::from_str(input).unwrap();

        assert_eq!(
            bar_type.instrument_id,
            InstrumentId::from("BTCUSDT-PERP.BINANCE")
        );
        assert_eq!(
            bar_type.spec,
            BarSpecification {
                step: 1,
                aggregation: BarAggregation::Minute,
                price_type: PriceType::Last,
            }
        );
        assert_eq!(bar_type.aggregation_source, AggregationSource::External);
    }

    #[rstest]
    fn test_bar_type_parse_invalid_token_pos_0() {
        let input = "BTCUSDT-PERP-1-MINUTE-LAST-INTERNAL";
        let result = BarType::from_str(input);

        assert_eq!(
            result.unwrap_err().to_string(),
            format!("Error parsing `BarType` from '{input}', invalid token: 'BTCUSDT-PERP' at position 0")
        );
    }

    #[rstest]
    fn test_bar_type_parse_invalid_token_pos_1() {
        let input = "BTCUSDT-PERP.BINANCE-INVALID-MINUTE-LAST-INTERNAL";
        let result = BarType::from_str(input);

        assert_eq!(
            result.unwrap_err().to_string(),
            format!(
                "Error parsing `BarType` from '{input}', invalid token: 'INVALID' at position 1"
            )
        );
    }

    #[rstest]
    fn test_bar_type_parse_invalid_token_pos_2() {
        let input = "BTCUSDT-PERP.BINANCE-1-INVALID-LAST-INTERNAL";
        let result = BarType::from_str(input);

        assert_eq!(
            result.unwrap_err().to_string(),
            format!(
                "Error parsing `BarType` from '{input}', invalid token: 'INVALID' at position 2"
            )
        );
    }

    #[rstest]
    fn test_bar_type_parse_invalid_token_pos_3() {
        let input = "BTCUSDT-PERP.BINANCE-1-MINUTE-INVALID-INTERNAL";
        let result = BarType::from_str(input);

        assert_eq!(
            result.unwrap_err().to_string(),
            format!(
                "Error parsing `BarType` from '{input}', invalid token: 'INVALID' at position 3"
            )
        );
    }

    #[rstest]
    fn test_bar_type_parse_invalid_token_pos_4() {
        let input = "BTCUSDT-PERP.BINANCE-1-MINUTE-BID-INVALID";
        let result = BarType::from_str(input);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            format!(
                "Error parsing `BarType` from '{input}', invalid token: 'INVALID' at position 4"
            )
        );
    }

    #[rstest]
    fn test_bar_type_equality() {
        let instrument_id1 = InstrumentId {
            symbol: Symbol::new("AUD/USD").unwrap(),
            venue: Venue::new("SIM").unwrap(),
        };
        let instrument_id2 = InstrumentId {
            symbol: Symbol::new("GBP/USD").unwrap(),
            venue: Venue::new("SIM").unwrap(),
        };
        let bar_spec = BarSpecification {
            step: 1,
            aggregation: BarAggregation::Minute,
            price_type: PriceType::Bid,
        };
        let bar_type1 = BarType {
            instrument_id: instrument_id1.clone(),
            spec: bar_spec.clone(),
            aggregation_source: AggregationSource::External,
        };
        let bar_type2 = BarType {
            instrument_id: instrument_id1,
            spec: bar_spec.clone(),
            aggregation_source: AggregationSource::External,
        };
        let bar_type3 = BarType {
            instrument_id: instrument_id2,
            spec: bar_spec,
            aggregation_source: AggregationSource::External,
        };
        assert_eq!(bar_type1, bar_type1);
        assert_eq!(bar_type1, bar_type2);
        assert_ne!(bar_type1, bar_type3);
    }

    #[rstest]
    fn test_bar_type_comparison() {
        let instrument_id1 = InstrumentId {
            symbol: Symbol::new("AUD/USD").unwrap(),
            venue: Venue::new("SIM").unwrap(),
        };

        let instrument_id2 = InstrumentId {
            symbol: Symbol::new("GBP/USD").unwrap(),
            venue: Venue::new("SIM").unwrap(),
        };
        let bar_spec = BarSpecification {
            step: 1,
            aggregation: BarAggregation::Minute,
            price_type: PriceType::Bid,
        };
        let bar_type1 = BarType {
            instrument_id: instrument_id1.clone(),
            spec: bar_spec.clone(),
            aggregation_source: AggregationSource::External,
        };
        let bar_type2 = BarType {
            instrument_id: instrument_id1,
            spec: bar_spec.clone(),
            aggregation_source: AggregationSource::External,
        };
        let bar_type3 = BarType {
            instrument_id: instrument_id2,
            spec: bar_spec,
            aggregation_source: AggregationSource::External,
        };

        assert!(bar_type1 <= bar_type2);
        assert!(bar_type1 < bar_type3);
        assert!(bar_type3 > bar_type1);
        assert!(bar_type3 >= bar_type1);
    }

    #[rstest]
    fn test_bar_equality() {
        let instrument_id = InstrumentId {
            symbol: Symbol::new("AUDUSD").unwrap(),
            venue: Venue::new("SIM").unwrap(),
        };
        let bar_spec = BarSpecification {
            step: 1,
            aggregation: BarAggregation::Minute,
            price_type: PriceType::Bid,
        };
        let bar_type = BarType {
            instrument_id,
            spec: bar_spec,
            aggregation_source: AggregationSource::External,
        };
        let bar1 = Bar {
            bar_type: bar_type.clone(),
            open: Price::from("1.00001"),
            high: Price::from("1.00004"),
            low: Price::from("1.00002"),
            close: Price::from("1.00003"),
            volume: Quantity::from("100000"),
            ts_event: 0,
            ts_init: 0,
        };

        let bar2 = Bar {
            bar_type,
            open: Price::from("1.00000"),
            high: Price::from("1.00004"),
            low: Price::from("1.00002"),
            close: Price::from("1.00003"),
            volume: Quantity::from("100000"),
            ts_event: 0,
            ts_init: 0,
        };
        assert_eq!(bar1, bar1);
        assert_ne!(bar1, bar2);
    }

    #[rstest]
    fn test_as_dict() {
        pyo3::prepare_freethreaded_python();

        let bar = create_stub_bar();

        Python::with_gil(|py| {
            let dict_string = bar.as_dict(py).unwrap().to_string();
            let expected_string = r#"{'type': 'Bar', 'bar_type': 'AUDUSD.SIM-1-MINUTE-BID-EXTERNAL', 'open': '1.00001', 'high': '1.00004', 'low': '1.00002', 'close': '1.00003', 'volume': '100000', 'ts_event': 0, 'ts_init': 1}"#;
            assert_eq!(dict_string, expected_string);
        });
    }

    #[rstest]
    fn test_as_from_dict() {
        pyo3::prepare_freethreaded_python();

        let bar = create_stub_bar();

        Python::with_gil(|py| {
            let dict = bar.as_dict(py).unwrap();
            let parsed = Bar::from_dict(py, dict).unwrap();
            assert_eq!(parsed, bar);
        });
    }

    #[rstest]
    fn test_from_pyobject() {
        pyo3::prepare_freethreaded_python();
        let bar = create_stub_bar();

        Python::with_gil(|py| {
            let bar_pyobject = bar.into_py(py);
            let parsed_bar = Bar::from_pyobject(bar_pyobject.as_ref(py)).unwrap();
            assert_eq!(parsed_bar, bar);
        });
    }

    #[rstest]
    fn test_json_serialization() {
        let bar = create_stub_bar();
        let serialized = bar.as_json_bytes().unwrap();
        let deserialized = Bar::from_json_bytes(serialized).unwrap();
        assert_eq!(deserialized, bar);
    }

    #[rstest]
    fn test_msgpack_serialization() {
        let bar = create_stub_bar();
        let serialized = bar.as_msgpack_bytes().unwrap();
        let deserialized = Bar::from_msgpack_bytes(serialized).unwrap();
        assert_eq!(deserialized, bar);
    }
}
