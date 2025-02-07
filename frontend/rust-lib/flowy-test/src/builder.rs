use flowy_user::entities::UserProfile;
use lib_dispatch::prelude::{EventDispatcher, EventResponse, FromBytes, ModuleRequest, StatusCode, ToBytes};
use std::{
    fmt::{Debug, Display},
    hash::Hash,
};

use crate::FlowyTestSDK;
use lib_dispatch::prelude::*;

use flowy_sdk::*;
use flowy_user::errors::UserError;
use flowy_workspace::errors::WorkspaceError;
use std::{convert::TryFrom, marker::PhantomData, sync::Arc};

pub type FlowyWorkspaceTest = Builder<WorkspaceError>;
impl FlowyWorkspaceTest {
    pub fn new(sdk: FlowyTestSDK) -> Self { Builder::test(TestContext::new(sdk)) }
}

pub type UserTest = Builder<UserError>;
impl UserTest {
    pub fn new(sdk: FlowyTestSDK) -> Self { Builder::test(TestContext::new(sdk)) }
    pub fn user_profile(&self) -> &Option<UserProfile> { &self.user_profile }
}

#[derive(Clone)]
pub struct Builder<E> {
    context: TestContext,
    user_profile: Option<UserProfile>,
    err_phantom: PhantomData<E>,
}

impl<E> Builder<E>
where
    E: FromBytes + Debug,
{
    pub(crate) fn test(context: TestContext) -> Self {
        Self {
            context,
            user_profile: None,
            err_phantom: PhantomData,
        }
    }

    pub fn request<P>(mut self, payload: P) -> Self
    where
        P: ToBytes,
    {
        match payload.into_bytes() {
            Ok(bytes) => {
                let module_request = self.get_request();
                self.context.request = Some(module_request.payload(bytes))
            },
            Err(e) => {
                log::error!("Set payload failed: {:?}", e);
            },
        }
        self
    }

    pub fn event<Event>(mut self, event: Event) -> Self
    where
        Event: Eq + Hash + Debug + Clone + Display,
    {
        self.context.request = Some(ModuleRequest::new(event));
        self
    }

    pub fn sync_send(mut self) -> Self {
        let request = self.get_request();
        let resp = EventDispatcher::sync_send(self.dispatch(), request);
        self.context.response = Some(resp);
        self
    }

    pub async fn async_send(mut self) -> Self {
        let request = self.get_request();
        let resp = EventDispatcher::async_send(self.dispatch(), request).await;
        self.context.response = Some(resp);
        self
    }

    pub fn parse<R>(self) -> R
    where
        R: FromBytes,
    {
        let response = self.get_response();
        match response.parse::<R, E>() {
            Ok(Ok(data)) => data,
            Ok(Err(e)) => {
                panic!("parse failed: {:?}", e)
            },
            Err(e) => panic!("Internal error: {:?}", e),
        }
    }

    pub fn error(self) -> E {
        let response = self.get_response();
        assert_eq!(response.status_code, StatusCode::Err);
        <Data<E>>::try_from(response.payload).unwrap().into_inner()
    }

    pub fn assert_error(self) -> Self {
        // self.context.assert_error();
        self
    }

    pub fn assert_success(self) -> Self {
        // self.context.assert_success();
        self
    }

    pub fn sdk(&self) -> FlowySDK { self.context.sdk.clone() }

    fn dispatch(&self) -> Arc<EventDispatcher> { self.context.sdk.dispatcher() }

    fn get_response(&self) -> EventResponse {
        self.context
            .response
            .as_ref()
            .expect("must call sync_send first")
            .clone()
    }

    fn get_request(&mut self) -> ModuleRequest { self.context.request.take().expect("must call event first") }
}

#[derive(Clone)]
pub struct TestContext {
    sdk: FlowyTestSDK,
    request: Option<ModuleRequest>,
    response: Option<EventResponse>,
}

impl TestContext {
    pub fn new(sdk: FlowyTestSDK) -> Self {
        Self {
            sdk,
            request: None,
            response: None,
        }
    }
}
