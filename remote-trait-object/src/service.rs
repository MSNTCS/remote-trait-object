// Copyright 2020 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

pub mod id;
pub mod remote;
pub mod serde_support;

use crate::forwarder::ServiceObjectId;
use crate::port::Port;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Weak};

pub type MethodId = u32;

/// This represents transportable identifier of the service object
/// and should be enough to construct a handle along with the pointer to the port
/// which this service belong to
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Copy)]
pub struct HandleToExchange(pub(crate) ServiceObjectId);

// TODO: Remove this
impl HandleToExchange {
    pub fn new_singleton() -> Self {
        HandleToExchange(0)
    }
}

/// Remote service will carry this.
#[derive(Debug)]
pub struct Handle {
    pub id: ServiceObjectId,
    pub port: Weak<dyn Port>,
}

impl Handle {
    /// You should not call this! This is for the macro.
    pub fn careful_new(imported_id: HandleToExchange, port: Weak<dyn Port>) -> Self {
        Handle {
            id: imported_id.0,
            port,
        }
    }
}

/// Exporter sides's interface to the service object. This will be implemented
/// by each service trait's unique wrapper in the macro
pub trait Dispatch: Send + Sync {
    fn dispatch_and_call(&self, method: MethodId, args: &[u8]) -> Vec<u8>;
}

impl<F> Dispatch for F
where
    F: Fn(MethodId, &[u8]) -> Vec<u8> + Send + Sync,
{
    fn dispatch_and_call(&self, method: MethodId, args: &[u8]) -> Vec<u8> {
        self(method, args)
    }
}

/// These two traits are associated with some specific service trait.
/// These tratis will be implement by `dyn ServiceTrait` where `T = dyn ServiceTrait` as well.
/// Macro will implement this trait with the target(expanding) service trait.
pub trait ImportService<T: ?Sized + Service> {
    fn import(port: Weak<dyn Port>, handle: HandleToExchange) -> Arc<T>;
}

pub trait ExportService<T: ?Sized + Service> {
    fn export(port: Weak<dyn Port>, object: Arc<T>) -> HandleToExchange;
}

#[macro_export]
macro_rules! export_service {
    ($service_trait: path, $context: expr, $service_object: expr) => {{
        let port = $context.get_port();
        <dyn $service_trait as remote_trait_object::ExportService<dyn $service_trait>>::export(port, $service_object)
    }};
}

#[macro_export]
macro_rules! import_service {
    ($service_trait: path, $context: expr, $handle: expr) => {{
        let port = $context.get_port();
        <dyn $service_trait as remote_trait_object::ImportService<dyn $service_trait>>::import(port, $handle)
    }};
}

/// All service trait must implement this.
/// This trait serves as a mere marker trait with two bounds
pub trait Service: Send + Sync {}
