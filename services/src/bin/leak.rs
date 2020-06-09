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
struct Leak {
    author: Address,
    public_description: String,
    message: String,
    message_release_time: u64
}


/// The core abstraction is a Leak.
///
/// Leaks have a `public_description`, which is always visible.
///
/// They also have a `message`, which is released at `message_release_time`.
///
/// The author of the original leak---and only this author---can
/// `change_release_time(new_time)`. (Note: The author can only change the time
/// of an unreleased message; an already-released message cannot have its time
/// changed).
///
impl Leak {
    pub fn new(ctx: &Context,
               public_description: String,
               message: String,
               message_release_time: u64) -> Self {
        Self {
            author: ctx.sender(),
            public_description,
            message,
            message_release_time,
        }
    }

    /// Leak the message at some different time. Only the message's author can do this.
    pub fn change_release_time(&mut self, ctx: &Context, new_time: u64) -> Result<()> {
        // If the caller is not the message author,
        // They get an error.
        if !(&self.author == &ctx.sender()) {
            return Err("Sender does not have permission to make message public.".to_string());
        }
        // If the caller IS not the message author,
        // but the message has already been released,
        // They get an error.
        //
        // This prevents authors from getting a false sense of security
        // By making confidential an already-released message.
        else if now() > self.message_release_time {
            return Err("Cannot update release time of already-released message.".to_string());
        }
        self.message_release_time = new_time;
        Ok(())
    }

    /// Get the message. Anyone can do this *if* the time the message becomes public is in the past.
    pub fn message(&self, _ctx: &Context) -> Result<String> {
        // If it's not yet release time,
        // return an error.
        if now() < self.message_release_time {
            return Err("Message is not yet released.".to_string());
        }
        Ok(self.message.clone())
    }

    /// Get the public description of the leak. Anyone can do this.
    pub fn get_public_description (&self, _ctx: &Context) -> Result<String> {
        Ok(self.public_description.clone())
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
            Leak::new(&author_ctx,
                         description.to_string(),
                         message.to_string(),
                         release_time);

        // Neither Author nor Viewer should be able to read message before it is released
        assert!(release.message(&author_ctx).is_err());
        assert!(release.message(&viewer_ctx).is_err());

        // Both author and viewer should be able to read the public description
        // of the message, even if the message itself hasn't been released
        assert_eq!(release.get_public_description(&author_ctx).unwrap(), description);
    }

    #[test]
    fn can_access_past_message() {
        let (_author_address, author_ctx) = create_account_ctx();
        let (_viewer_address, viewer_ctx) = create_account_ctx();

        let description = "My big news";
        let message = "I'm in love with kimchi.";
        let release_time = now()-100000;

        let release=
            Leak::new(&author_ctx,
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
            Leak::new(&author_ctx,
                         description.to_string(),
                         message.to_string(),
                         release_time);

        // Neither Author nor Viewer nor Leakr should be able to read message before it is released
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

    #[test]
    fn author_cannot_change_message_release_time_if_message_already_public() {
        let (_author_address, author_ctx) = create_account_ctx();

        let description = "My big news";
        let message = "I'm in love with kimchi.";
        let release_time = now()-100000; // some time in the past

        let mut release=
            Leak::new(&author_ctx,
                         description.to_string(),
                         message.to_string(),
                         release_time);

        let new_release_time = now()+100000; // some time in the future
        // should NOT be allowed to make this change
        release.change_release_time(&author_ctx, new_release_time).unwrap_err();
    }
}


fn main() {
    oasis_std::service!(Leak);
}

