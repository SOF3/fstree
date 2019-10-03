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

use std::fmt::{self, Debug, Display};
use std::{error, io};

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
}

impl Error {
    pub fn as_err(&self) -> &(dyn error::Error + Send + Sync + 'static) {
        match self {
            Error::Io(err) => err,
        }
    }
}

impl From<io::Error> for Error {
    fn from(inner: io::Error) -> Self {
        Error::Io(inner)
    }
}

pub fn make_err<E>(err: E) -> Error
where
    E: Into<Box<dyn error::Error + Send + Sync + 'static>>,
{
    let err = io::Error::new(io::ErrorKind::Other, err);
    err.into()
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = self.as_err();
        Display::fmt(err, f)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.as_err().source()
    }
}
