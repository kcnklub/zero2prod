use validator::validate_email;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(input: String) -> Result<SubscriberEmail, String> {
        if validate_email(&input) {
            Ok(Self(input))
        } else {
            Err(format!("{} is not a valid email!", input))
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use claims::assert_err;
    use fake::{faker::internet::en::SafeEmail, Fake};

    use super::SubscriberEmail;

    #[test]
    fn empty_string_is_rejected() {
        let input = "".to_string();
        assert_err!(SubscriberEmail::parse(input));
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let input = "testtest.com".to_string();
        assert_err!(SubscriberEmail::parse(input));
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        let input = "@test.com".to_string();
        assert_err!(SubscriberEmail::parse(input));
    }

    #[test]
    fn valid_emails_are_parsed_successfully() {
        let email = SafeEmail().fake();
        claims::assert_ok!(SubscriberEmail::parse(email));
    }
}
