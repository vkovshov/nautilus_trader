// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2021 Nautech Systems Pty Ltd. All rights reserved.
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

#[cfg(test)]
mod tests {
    use nautilus_order_book::entry::OrderBookEntry;

    #[test]
    fn instantiate() {
        let entry = OrderBookEntry { price: 10500.0, qty: 510.0, update_id: 1 };

        assert_eq!(10500.0, entry.price);
        assert_eq!(510.0, entry.qty);
        assert_eq!(1, entry.update_id);
    }

    #[test]
    fn update() {
        let mut entry = OrderBookEntry { price: 10500.0, qty: 510.0, update_id: 1 };
        entry.update(600.0, 2);

        assert_eq!(10500.0, entry.price);
        assert_eq!(600.0, entry.qty);
        assert_eq!(2, entry.update_id);
    }
}
