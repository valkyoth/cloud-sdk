use cloud_sdk::Method;
use cloud_sdk::transport::{BlockingTransport, RequestTarget, ResponseBuffer, TransportRequest};

use crate::{
    ActionFixture, ActionScript, ActionState, DynamicMockTransport, FixtureBody, PaginationFixture,
    PaginationScript, RequestRecordSlot, ResponseFixture, ScenarioScriptError,
};

fn body() -> FixtureBody<'static> {
    let Ok(body) = FixtureBody::new(b"{}") else {
        unreachable!()
    };
    body
}

#[test]
fn pagination_script_validates_and_serves_every_page() {
    let (Ok(first), Ok(second)) = (
        PaginationFixture::new(1, 1, 2, 2),
        PaginationFixture::new(2, 1, 2, 2),
    ) else {
        unreachable!("testkit security fixture construction failed");
    };
    let fixtures = [
        ResponseFixture::paginated(body(), first),
        ResponseFixture::paginated(body(), second),
    ];
    let Ok(script) = PaginationScript::new(&fixtures) else {
        unreachable!("testkit security fixture construction failed");
    };
    let slots = [const { RequestRecordSlot::new() }; 2];
    let Ok(transport) = DynamicMockTransport::new(script, &slots) else {
        unreachable!("testkit security fixture construction failed");
    };
    let Ok(target) = RequestTarget::new("/pages") else {
        unreachable!("testkit security fixture construction failed");
    };
    for _ in 0..2 {
        let mut output = [0_u8; 2];
        let mut headers = [0_u8; 32];
        let mut response = ResponseBuffer::new(&mut output, 2, &mut headers);
        assert!(
            transport
                .send(
                    TransportRequest::new(Method::Get, target),
                    response.writer()
                )
                .is_ok()
        );
    }
    assert_eq!(transport.recorded(), 2);
}

#[test]
fn pagination_script_rejects_gaps_and_incomplete_sequences() {
    let (Ok(second), Ok(third)) = (
        PaginationFixture::new(2, 1, 3, 3),
        PaginationFixture::new(3, 1, 3, 3),
    ) else {
        unreachable!("testkit security fixture construction failed");
    };
    let gap = [
        ResponseFixture::paginated(body(), second),
        ResponseFixture::paginated(body(), third),
    ];
    assert!(matches!(
        PaginationScript::new(&gap),
        Err(ScenarioScriptError::InvalidPageSequence)
    ));
    let Ok(first) = PaginationFixture::new(1, 1, 3, 3) else {
        unreachable!("testkit security fixture construction failed");
    };
    let incomplete = [ResponseFixture::paginated(body(), first)];
    assert!(matches!(
        PaginationScript::new(&incomplete),
        Err(ScenarioScriptError::PaginationDidNotFinish)
    ));
}

#[test]
fn action_script_requires_monotonic_progress_and_final_terminal_state() {
    let (Ok(running), Ok(success)) = (
        ActionFixture::new(ActionState::Running, 40),
        ActionFixture::new(ActionState::Success, 100),
    ) else {
        unreachable!("testkit security fixture construction failed");
    };
    let valid = [
        ResponseFixture::action(body(), running),
        ResponseFixture::action(body(), success),
    ];
    assert!(ActionScript::new(&valid).is_ok());

    let Ok(regressed) = ActionFixture::new(ActionState::Running, 20) else {
        unreachable!("testkit security fixture construction failed");
    };
    let regression = [
        ResponseFixture::action(body(), running),
        ResponseFixture::action(body(), regressed),
        ResponseFixture::action(body(), success),
    ];
    assert!(matches!(
        ActionScript::new(&regression),
        Err(ScenarioScriptError::ActionProgressDecreased)
    ));
    let unfinished = [ResponseFixture::action(body(), running)];
    assert!(matches!(
        ActionScript::new(&unfinished),
        Err(ScenarioScriptError::ActionDidNotFinish)
    ));
}
