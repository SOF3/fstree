// fstree
// Copyright (C) SOFe
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affer General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use std::fmt::Display;

use wasm_bindgen::JsValue;

pub type Result<T = (), E = JsValue> = std::result::Result<T, E>;

#[allow(dead_code)]
pub fn make_err<E>(err: E) -> JsValue
where
    E: Display,
{
    JsValue::from_str(&err.to_string())
}
