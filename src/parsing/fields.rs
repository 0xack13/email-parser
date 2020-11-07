use crate::parsing::time::*;
use crate::prelude::*;

#[derive(Debug)]
pub enum TraceField<'a> {
    Date(DateTime),
    From(Vec<(Option<Vec<String<'a>>>, (String<'a>, String<'a>))>),
    Sender(Mailbox<'a>),
    To(Vec<Address<'a>>),
    Cc(Vec<Address<'a>>),
    Bcc(Vec<Address<'a>>),
    MessageId((String<'a>, String<'a>)),
}

#[derive(Debug)]
pub enum Field<'a> {
    Date(DateTime),
    From(Vec<(Option<Vec<String<'a>>>, (String<'a>, String<'a>))>),
    Sender(Mailbox<'a>),
    ReplyTo(Vec<Address<'a>>),
    To(Vec<Address<'a>>),
    Cc(Vec<Address<'a>>),
    Bcc(Vec<Address<'a>>),
    MessageId((String<'a>, String<'a>)),
    InReplyTo(Vec<(String<'a>, String<'a>)>),
    References(Vec<(String<'a>, String<'a>)>),
    Subject(String<'a>),
    Comments(String<'a>),
    Keywords(Vec<Vec<String<'a>>>),
    Trace {
        return_path: Option<Option<(String<'a>, String<'a>)>>,
        received: Vec<(Vec<ReceivedToken<'a>>, DateTime)>,
        fields: Vec<TraceField<'a>>,
    },
    Unknown {
        name: String<'a>,
        value: String<'a>,
    },
}

pub fn fields(mut input: &[u8]) -> Res<Vec<Field>> {
    let mut fields: Vec<Field> = Vec::new();

    while let Ok((new_input, trace)) = trace(input) {
        input = new_input;
        let mut trace_fields = Vec::new();

        while let Ok((new_input, new_result)) = match_parsers(
            input,
            &mut [
                |i| resent_date(i).map(|(i, v)| (i, TraceField::Date(v))),
                |i| resent_from(i).map(|(i, v)| (i, TraceField::From(v))),
                |i| resent_sender(i).map(|(i, v)| (i, TraceField::Sender(v))),
                |i| resent_to(i).map(|(i, v)| (i, TraceField::To(v))),
                |i| resent_cc(i).map(|(i, v)| (i, TraceField::Cc(v))),
                |i| resent_bcc(i).map(|(i, v)| (i, TraceField::Bcc(v))),
                |i| resent_message_id(i).map(|(i, v)| (i, TraceField::MessageId(v))),
            ][..],
        ) {
            input = new_input;
            trace_fields.push(new_result);
        }

        // TODO optional fields

        fields.push(Field::Trace {
            return_path: trace.0,
            received: trace.1,
            fields: trace_fields,
        });
    }

    while let Ok((new_input, field)) = match_parsers(
        input,
        &mut [
            |i| date(i).map(|(i, v)| (i, Field::Date(v))),
            |i| from(i).map(|(i, v)| (i, Field::From(v))),
            |i| sender(i).map(|(i, v)| (i, Field::Sender(v))),
            |i| reply_to(i).map(|(i, v)| (i, Field::ReplyTo(v))),
            |i| to(i).map(|(i, v)| (i, Field::To(v))),
            |i| cc(i).map(|(i, v)| (i, Field::Cc(v))),
            |i| bcc(i).map(|(i, v)| (i, Field::Bcc(v))),
            |i| message_id(i).map(|(i, v)| (i, Field::MessageId(v))),
            |i| in_reply_to(i).map(|(i, v)| (i, Field::InReplyTo(v))),
            |i| references(i).map(|(i, v)| (i, Field::References(v))),
            |i| subject(i).map(|(i, v)| (i, Field::Subject(v))),
            |i| comments(i).map(|(i, v)| (i, Field::Comments(v))),
            |i| keywords(i).map(|(i, v)| (i, Field::Keywords(v))),
            |i| unknown(i).map(|(i, (name, value))| (i, Field::Unknown { name, value })),
        ][..],
    ) {
        input = new_input;
        fields.push(field);
    }

    Ok((input, fields))
}

pub fn date(input: &[u8]) -> Res<DateTime> {
    let (input, ()) = tag_no_case(input, b"Date:", b"dATE:")?;
    let (input, date_time) = date_time(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, date_time))
}

pub fn from(input: &[u8]) -> Res<Vec<(Option<Vec<String>>, (String, String))>> {
    let (input, ()) = tag_no_case(input, b"From:", b"fROM:")?;
    let (input, mailbox_list) = mailbox_list(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, mailbox_list))
}

pub fn sender(input: &[u8]) -> Res<Mailbox> {
    let (input, ()) = tag_no_case(input, b"Sender:", b"sENDER:")?;
    let (input, mailbox) = mailbox(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, mailbox))
}

pub fn reply_to(input: &[u8]) -> Res<Vec<Address>> {
    let (input, ()) = tag_no_case(input, b"Reply-To:", b"rEPLY-tO:")?;
    let (input, mailbox) = address_list(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, mailbox))
}

pub fn to(input: &[u8]) -> Res<Vec<Address>> {
    let (input, ()) = tag_no_case(input, b"To:", b"tO:")?;
    let (input, mailbox) = address_list(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, mailbox))
}

pub fn cc(input: &[u8]) -> Res<Vec<Address>> {
    let (input, ()) = tag_no_case(input, b"Cc:", b"cC:")?;
    let (input, mailbox) = address_list(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, mailbox))
}

pub fn bcc(input: &[u8]) -> Res<Vec<Address>> {
    let (input, ()) = tag_no_case(input, b"Bcc:", b"bCC:")?;
    let (input, mailbox) = if let Ok((input, list)) = address_list(input) {
        (input, list)
    } else if let Ok((input, _cfws)) = cfws(input) {
        (input, Vec::new())
    } else {
        return Err(Error::Known("Invalid bcc field"));
    };
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, mailbox))
}

pub fn message_id(input: &[u8]) -> Res<(String, String)> {
    let (input, ()) = tag_no_case(input, b"Message-ID:", b"mESSAGE-id:")?;
    let (input, id) = crate::parsing::address::message_id(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, id))
}

pub fn in_reply_to(input: &[u8]) -> Res<Vec<(String, String)>> {
    let (input, ()) = tag_no_case(input, b"In-Reply-To:", b"iN-rEPLY-tO:")?;
    let (input, ids) = many1(input, crate::parsing::address::message_id)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, ids))
}

pub fn references(input: &[u8]) -> Res<Vec<(String, String)>> {
    let (input, ()) = tag_no_case(input, b"References:", b"rEFERENCES:")?;
    let (input, ids) = many1(input, crate::parsing::address::message_id)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, ids))
}

pub fn subject(input: &[u8]) -> Res<String> {
    let (input, ()) = tag_no_case(input, b"Subject:", b"sUBJECT:")?;
    let (input, subject) = unstructured(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, subject))
}

pub fn comments(input: &[u8]) -> Res<String> {
    let (input, ()) = tag_no_case(input, b"Comments:", b"cOMMENTS:")?;
    let (input, comments) = unstructured(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, comments))
}

pub fn keywords(input: &[u8]) -> Res<Vec<Vec<String>>> {
    let (input, ()) = tag_no_case(input, b"Keywords:", b"kEYWORDS:")?;

    let mut keywords = Vec::new();
    let (mut input, first_keyword) = phrase(input)?;
    keywords.push(first_keyword);

    while let Ok((new_input, new_keyword)) = prefixed(input, phrase, ",") {
        input = new_input;
        keywords.push(new_keyword);
    }

    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, keywords))
}

pub fn resent_date(input: &[u8]) -> Res<DateTime> {
    let (input, ()) = tag_no_case(input, b"Resent-", b"rESENT-")?;
    let (input, date) = date(input)?;

    Ok((input, date))
}

pub fn resent_from(input: &[u8]) -> Res<Vec<(Option<Vec<String>>, (String, String))>> {
    let (input, ()) = tag_no_case(input, b"Resent-", b"rESENT-")?;
    let (input, from) = from(input)?;

    Ok((input, from))
}

pub fn resent_sender(input: &[u8]) -> Res<Mailbox> {
    let (input, ()) = tag_no_case(input, b"Resent-", b"rESENT-")?;
    let (input, sender) = sender(input)?;

    Ok((input, sender))
}

pub fn resent_to(input: &[u8]) -> Res<Vec<Address>> {
    let (input, ()) = tag_no_case(input, b"Resent-", b"rESENT-")?;
    let (input, to) = to(input)?;

    Ok((input, to))
}

pub fn resent_cc(input: &[u8]) -> Res<Vec<Address>> {
    let (input, ()) = tag_no_case(input, b"Resent-", b"rESENT-")?;
    let (input, cc) = cc(input)?;

    Ok((input, cc))
}

pub fn resent_bcc(input: &[u8]) -> Res<Vec<Address>> {
    let (input, ()) = tag_no_case(input, b"Resent-", b"rESENT-")?;
    let (input, bcc) = bcc(input)?;

    Ok((input, bcc))
}

pub fn resent_message_id(input: &[u8]) -> Res<(String, String)> {
    let (input, ()) = tag_no_case(input, b"Resent-", b"rESENT-")?;
    let (input, id) = message_id(input)?;

    Ok((input, id))
}

pub fn return_path(input: &[u8]) -> Res<Option<(String, String)>> {
    fn empty_path(input: &[u8]) -> Res<()> {
        let (input, _cfws) = optional(input, cfws);
        let (input, ()) = tag(input, b"<")?;
        let (input, _cfws) = optional(input, cfws);
        let (input, ()) = tag(input, b">")?;
        let (input, _cfws) = optional(input, cfws);
        Ok((input, ()))
    }

    let (input, ()) = tag_no_case(input, b"Return-Path:", b"rETURN-pATH:")?;
    let (input, addr) = match_parsers(
        input,
        &mut [
            (|i| angle_addr(i).map(|(i, v)| (i, Some(v))))
                as fn(input: &[u8]) -> Res<Option<(String, String)>>,
            (|i| empty_path(i).map(|(i, _)| (i, None)))
                as fn(input: &[u8]) -> Res<Option<(String, String)>>,
        ][..],
    )?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, addr))
}

#[derive(Debug)]
pub enum ReceivedToken<'a> {
    Word(String<'a>),
    Addr((String<'a>, String<'a>)),
    Domain(String<'a>),
}

pub fn received(input: &[u8]) -> Res<(Vec<ReceivedToken>, DateTime)> {
    let (input, ()) = tag_no_case(input, b"Received:", b"rECEIVED:")?;
    let (input, received_tokens) = many(input, |input| {
        if let Ok((word_input, word)) = word(input) {
            if let Ok((domain_input, domain)) = domain(input) {
                if domain.len() > word.len() {
                    return Ok((domain_input, ReceivedToken::Domain(domain)));
                }
            }
            Ok((word_input, ReceivedToken::Word(word)))
        } else if let Ok((input, addr)) = angle_addr(input) {
            Ok((input, ReceivedToken::Addr(addr)))
        } else if let Ok((input, addr)) = addr_spec(input) {
            Ok((input, ReceivedToken::Addr(addr)))
        } else if let Ok((input, domain)) = domain(input) {
            Ok((input, ReceivedToken::Domain(domain)))
        } else {
            Err(Error::Known("match error"))
        }
    })?;
    let (input, ()) = tag(input, b";")?;
    let (input, date_time) = date_time(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, (received_tokens, date_time)))
}

pub fn trace(
    input: &[u8],
) -> Res<(
    Option<Option<(String, String)>>,
    Vec<(Vec<ReceivedToken>, DateTime)>,
)> {
    let (input, return_path) = optional(input, return_path);
    let (input, received) = many1(input, received)?;

    Ok((input, (return_path, received)))
}

pub fn unknown(input: &[u8]) -> Res<(String, String)> {
    let (input, name) = take_while1(input, is_ftext)?;
    let (input, ()) = tag(input, b":")?;
    let (input, value) = unstructured(input)?;
    let (input, ()) = tag(input, b"\r\n")?;

    Ok((input, (name, value)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fields() {
        assert!(fields(b"To: Mubelotix <mubelotix@gmail.com>\r\nFrOm: Mubelotix <mubelotix@gmail.com>\r\n").unwrap().0.is_empty());
        //println!("{:#?}", fields(include_bytes!("../../mail.txt")).unwrap().1);
    }

    #[test]
    fn test_unknown_field() {
        assert_eq!(unknown(b"hidden-field:hidden message\r\n").unwrap().1.1, "hidden message");
        assert_eq!(unknown(b"hidden-field:hidden message\r\n").unwrap().1.0, "hidden-field");
    }

    #[test]
    fn test_trace() {
        assert!(return_path(b"Return-Path:<>\r\n").unwrap().1.is_none());
        assert_eq!(
            return_path(b"Return-Path:<mubelotix@gmail.com>\r\n")
                .unwrap()
                .1
                .unwrap()
                .0,
            "mubelotix"
        );

        assert!(matches!(received(b"Received:test<mubelotix@gmail.com>;5 May 2003 18:59:03 +0000\r\n").unwrap().1.0[0], ReceivedToken::Word(_)));
        assert!(matches!(received(b"Received:test<mubelotix@gmail.com>;5 May 2003 18:59:03 +0000\r\n").unwrap().1.0[1], ReceivedToken::Addr(_)));
        assert!(matches!(received(b"Received:mubelotix.dev;5 May 2003 18:59:03 +0000\r\n").unwrap().1.0[0], ReceivedToken::Domain(_)));

        assert!(trace(b"Return-Path:<>\r\nReceived:akala miam miam;5 May 2003 18:59:03 +0000\r\nReceived:mubelotix.dev;5 May 2003 18:59:03 +0000\r\n").unwrap().0.is_empty());
    }

    #[test]
    fn test_resent() {
        assert_eq!(
            resent_date(b"Resent-Date:5 May 2003 18:59:03 +0000\r\n")
                .unwrap()
                .1,
            (None, (5, Month::May, 2003), ((18, 59, 3), (true, 0, 0)))
        );
        assert_eq!(resent_from(b"Resent-FrOm: Mubelotix <mubelotix@gmail.com>\r\n").unwrap().1[0].1.0, "mubelotix");
        assert_eq!(resent_sender(b"Resent-sender: Mubelotix <mubelotix@gmail.com>\r\n").unwrap().1.1.1, "gmail.com");
        assert!(
            !resent_to(b"Resent-To: Mubelotix <mubelotix@gmail.com>\r\n")
                .unwrap()
                .1
                .is_empty()
        );
        assert!(
            !resent_cc(b"Resent-Cc: Mubelotix <mubelotix@gmail.com>\r\n")
                .unwrap()
                .1
                .is_empty()
        );
        assert!(
            !resent_bcc(b"Resent-Bcc: Mubelotix <mubelotix@gmail.com>\r\n")
                .unwrap()
                .1
                .is_empty()
        );
    }

    #[test]
    fn test_date() {
        assert_eq!(
            date(b"Date:5 May 2003 18:59:03 +0000\r\n").unwrap().1,
            (None, (5, Month::May, 2003), ((18, 59, 3), (true, 0, 0)))
        );
    }

    #[test]
    fn test_originators() {
        assert_eq!(from(b"FrOm: Mubelotix <mubelotix@gmail.com>\r\n").unwrap().1[0].1.0, "mubelotix");
        assert_eq!(sender(b"sender: Mubelotix <mubelotix@gmail.com>\r\n").unwrap().1.1.1, "gmail.com");
        assert_eq!(
            reply_to(b"Reply-to: Mubelotix <mubelotix@gmail.com>\r\n")
                .unwrap()
                .1
                .len(),
            1
        );
    }

    #[test]
    fn test_destination() {
        assert!(!to(b"To: Mubelotix <mubelotix@gmail.com>\r\n")
            .unwrap()
            .1
            .is_empty());
        assert!(!cc(b"Cc: Mubelotix <mubelotix@gmail.com>\r\n")
            .unwrap()
            .1
            .is_empty());
        assert!(!bcc(b"Bcc: Mubelotix <mubelotix@gmail.com>\r\n")
            .unwrap()
            .1
            .is_empty());
        assert!(bcc(b"Bcc: \r\n \r\n").unwrap().1.is_empty());
    }

    #[test]
    fn test_ids() {
        assert_eq!(message_id(b"Message-ID:<556100154@gmail.com>\r\n").unwrap().1.0, "556100154");
        assert_eq!(message_id(b"Message-ID:<556100154@gmail.com>\r\n").unwrap().1.1, "gmail.com");

        assert_eq!(
            references(b"References:<qzdzdq@qdz.com><dzdzjd@zdzdj.dz>\r\n")
                .unwrap()
                .1
                .len(),
            2
        );

        assert_eq!(
            in_reply_to(b"In-Reply-To:<eefes@qzd.fr><52@s.dz><adzd@zd.d>\r\n")
                .unwrap()
                .1
                .len(),
            3
        );
    }

    #[test]
    fn test_informational() {
        assert_eq!(
            subject(b"Subject:French school is boring\r\n")
                .unwrap()
                .1,
            "French school is boring"
        );
        assert_eq!(
            subject(b"Subject:Folding\r\n is slow\r\n").unwrap().1,
            "Folding is slow"
        );

        assert_eq!(
            comments(b"Comments:Rust is great\r\n").unwrap().1,
            "Rust is great"
        );

        assert_eq!(
            keywords(b"Keywords:rust parser fast zero copy,email rfc5322\r\n")
                .unwrap()
                .1
                .len(),
            2
        );
    }
}