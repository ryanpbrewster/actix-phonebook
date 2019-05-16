use actix::prelude::*;

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::Duration;

#[derive(Default)]
struct PhoneBookActor {
    registry: HashMap<Name, Addr<PersonActor>>,
}

struct PersonActor {
    name: Name,
    phone_book: Addr<PhoneBookActor>,
}

#[derive(Clone, Hash, Eq, PartialEq)]
struct Name(String);
impl ::std::fmt::Display for Name {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        f.write_str(&self.0)
    }
}

struct RegisterRequest {
    name: Name,
    addr: Addr<PersonActor>,
}

struct UnregisterRequest {
    name: Name,
}

struct PoisonPill;
impl Message for PoisonPill {
    type Result = ();
}

impl Actor for PhoneBookActor {
    type Context = Context<Self>;
}

impl Actor for PersonActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.phone_book
            .try_send(RegisterRequest {
                name: self.name.clone(),
                addr: ctx.address(),
            })
            .unwrap();

        let addr = ctx.address();
        ctx.run_later(Duration::from_millis(100), move |_, _| {
            addr.do_send(PoisonPill);
        });
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        self.phone_book
            .try_send(UnregisterRequest {
                name: self.name.clone(),
            })
            .unwrap();

        let num_children = {
            let mut h = DefaultHasher::new();
            self.name.hash(&mut h);
            h.finish() % 3
        };
        for i in 0..num_children {
            PersonActor {
                name: Name(format!("{}.{}", self.name, i)),
                phone_book: self.phone_book.clone(),
            }
            .start();
        }
    }
}
impl Handler<PoisonPill> for PersonActor {
    type Result = ();
    fn handle(&mut self, _msg: PoisonPill, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop();
    }
}

impl Message for RegisterRequest {
    type Result = ();
}
impl Message for UnregisterRequest {
    type Result = ();
}

impl Handler<RegisterRequest> for PhoneBookActor {
    type Result = ();

    fn handle(&mut self, msg: RegisterRequest, _ctx: &mut Self::Context) -> Self::Result {
        println!(
            "registering {} in the phone book, up to {}",
            msg.name,
            self.registry.len() + 1
        );
        self.registry.insert(msg.name, msg.addr);
    }
}

impl Handler<UnregisterRequest> for PhoneBookActor {
    type Result = ();

    fn handle(&mut self, msg: UnregisterRequest, _ctx: &mut Self::Context) -> Self::Result {
        println!(
            "removing {} in the phone book, down to {}",
            msg.name,
            self.registry.len() - 1
        );
        self.registry.remove(&msg.name);
    }
}

fn main() {
    // start system, this is required step
    System::run(|| {
        // start new actor
        let phone_book = PhoneBookActor::start_default();

        for i in 0..10 {
            PersonActor {
                name: Name(format!("{}", i)),
                phone_book: phone_book.clone(),
            }
            .start();
        }
    })
    .expect("run system");
}
