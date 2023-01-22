pub mod context;
pub mod states;

use machines_rs::{machine::Machine, state::State};

use self::{
    context::AccountContext,
    states::{new::New, States},
};

pub type AccountMachine = Machine<States, AccountContext>;

pub fn create_account_machine(initial_state: States) -> AccountMachine {
    let fsm = Machine::new(initial_state).state(
        States::New,
        State::new(New).transition(
            States::Created,
            |data| {
                return data.get_command().is_some()
                    && data.get_command().as_ref().unwrap().to_string() == "CreateAccount";
            },
            vec![],
        ),
    );
    return fsm;
}
