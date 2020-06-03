use oasis_std::{
    Service,
    abi::*,
    collections::Set, Address, Context, Event
};

type Result<T> = std::result::Result<T, String>;

pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}


#[derive(Service)]
struct Release {
    author: Address,
    description: String,
    message: String,
    message_becomes_public_time: u64
}


impl Release {
    pub fn new(ctx: &Context,
               description: String,
               message: String,
               message_becomes_public_time: u64) -> Self {
        Self {
            author: ctx.sender(),
            description,
            message,
            message_becomes_public_time,
        }
    }

    // TODO release_at(new_time: u64)

    pub fn message(&self, _ctx: &Context) -> Result<String> {
        if !(now() > self.message_becomes_public_time) {
            return Err("Message is not yet released.".to_string());
        }
        Ok(self.message.clone())
    }

}



#[cfg(test)]
mod tests {
    // This is required even in Rust 2018.
    // If omitted, rustc will not link in the testing
    // library and will produce a giant error message.
    extern crate oasis_test;

    use super::*;

    /// Creates a new account and a Context with the new account as the sender.
    fn create_account_ctx() -> (Address, Context) {
        let addr = oasis_test::create_account(0 /* initial balance */);
        let ctx = Context::default().with_sender(addr).with_gas(100_000);
        (addr, ctx)
    }


    /// Basic functionality; one releaser, requiring one vote to release.
    #[test]
    fn cannot_access_future_message() {
        let (author_address, author_ctx) = create_account_ctx();
        let (_viewer_address, viewer_ctx) = create_account_ctx();
        let (releaser_address, releaser_ctx) = create_account_ctx();

        let description = "My big news";
        let message = "I'm in love with kimchi.";
        let release_time = now()+100000;

        let mut release=
            Release::new(&author_ctx,
                         description.to_string(),
                         message.to_string(),
                         release_time);

        // Author nor Viewer nor Releaser should be able to read message before it is released
        assert!(release.message(&author_ctx).is_err());
        assert!(release.message(&viewer_ctx).is_err());
        assert!(release.message(&releaser_ctx).is_err());
    }

    #[test]
    fn can_access_past_message() {
        let (author_address, author_ctx) = create_account_ctx();
        let (_viewer_address, viewer_ctx) = create_account_ctx();
        let (releaser_address, releaser_ctx) = create_account_ctx();

        let description = "My big news";
        let message = "I'm in love with kimchi.";
        let release_time = now()-100000;

        let mut release=
            Release::new(&author_ctx,
                         description.to_string(),
                         message.to_string(),
                         release_time);

        // Author nor Viewer nor Releaser should be able to read message before it is released
        assert_eq!(release.message(&author_ctx).unwrap(), message);
        assert_eq!(release.message(&viewer_ctx).unwrap(), message);
        assert_eq!(release.message(&releaser_ctx).unwrap(), message);
    }

    // TODO heartbeat
}





#[derive(Service)]
struct Greeter {
    greeting: String,
    greeted: Set<String>,
    /* all fields must be (de)serializable, so remember to use
     * `#[derive(Serialize, Deserialize)]` when storing your own types */
}

impl Greeter {
    pub fn new(_ctx: &Context, greeting: String) -> Self {
        eprintln!("new confidential greeter has been deployed");
        Self {
            greeting,
            greeted: Set::new(), // a more efficient `Set` type for short-lived Wasm
        }
    }

    pub fn greet(&mut self, ctx: &Context, name: String) -> String {
        let greeting = format!("{} {}", self.greeting, name);
        Event::emit(&Greeted {
            from: &ctx.sender(),
            to: &name,
            time: now(),
        });
        self.greeted.insert(name);
        greeting
    }

    pub fn get_greeted(&self, _ctx: &Context) -> &Set<String> {
        &self.greeted
    }
}

#[derive(Serialize, Event)]
// Events are only emitted (never read), so no need to derive `Deserialize`.
pub struct Greeted<'a> {
    from: &'a Address,
    to: &'a String,
    time: u64,
}

fn main() {
    oasis_std::service!(Greeter);
}
