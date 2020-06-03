use oasis_std::{
    Service,
    Address,
    Context,
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
    message_release_time: u64
}


impl Release {
    pub fn new(ctx: &Context,
               description: String,
               message: String,
               message_release_time: u64) -> Self {
        Self {
            author: ctx.sender(),
            description,
            message,
            message_release_time,
        }
    }

    /// Release the message at some different time. Only the message's author can do this.
    pub fn change_release_time(&mut self, ctx: &Context, new_time: u64) -> Result<()> {
        dbg!("My release time is {}", self.message_release_time);
        // If the caller is not the message author,
        // They get an error.
        if !(&self.author == &ctx.sender()) {
            return Err("Sender does not have permission to make message public.".to_string());
        }
        self.message_release_time = new_time;
        dbg!("My release time is now {}", self.message_release_time);
        Ok(())
    }

    /// Get the message. Anyone can do this *if* the time the message becomes public is in the past.
    pub fn message(&self, _ctx: &Context) -> Result<String> {
        if !(now() > self.message_release_time) {
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
        let (_author_address, author_ctx) = create_account_ctx();
        let (_viewer_address, viewer_ctx) = create_account_ctx();

        let description = "My big news";
        let message = "I'm in love with kimchi.";
        let release_time = now()+100000;

        let release=
            Release::new(&author_ctx,
                         description.to_string(),
                         message.to_string(),
                         release_time);

        // Author nor Viewer nor Releaser should be able to read message before it is released
        assert!(release.message(&author_ctx).is_err());
        assert!(release.message(&viewer_ctx).is_err());
    }

    #[test]
    fn can_access_past_message() {
        let (_author_address, author_ctx) = create_account_ctx();
        let (_viewer_address, viewer_ctx) = create_account_ctx();

        let description = "My big news";
        let message = "I'm in love with kimchi.";
        let release_time = now()-100000;

        let release=
            Release::new(&author_ctx,
                         description.to_string(),
                         message.to_string(),
                         release_time);

        // Author and Viewer should be able to read message now
        assert_eq!(release.message(&author_ctx).unwrap(), message);
        assert_eq!(release.message(&viewer_ctx).unwrap(), message);
    }

    #[test]
    fn author_can_change_message_release_time() {
        let (_author_address, author_ctx) = create_account_ctx();
        let (_viewer_address, viewer_ctx) = create_account_ctx();

        let description = "My big news";
        let message = "I'm in love with kimchi.";
        let release_time = now()+100000; // some time in the future

        let mut release=
            Release::new(&author_ctx,
                         description.to_string(),
                         message.to_string(),
                         release_time);

        // Neither Author nor Viewer nor Releaser should be able to read message before it is released
        assert!(release.message(&author_ctx).is_err());
        assert!(release.message(&viewer_ctx).is_err());

        let new_release_time = now()-100000; // some time in the past

        // Only the author can extend release time.
        assert!(release.change_release_time(&viewer_ctx, new_release_time).is_err());
        release.change_release_time(&author_ctx, new_release_time).unwrap();

        // Author and Viewer should be able to read message now
        assert_eq!(release.message(&author_ctx).unwrap(), message);
        assert_eq!(release.message(&viewer_ctx).unwrap(), message);
    }

}


fn main() {
    oasis_std::service!(Release);
}

