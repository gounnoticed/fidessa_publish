extern crate curl;
extern crate chrono;
extern crate pushover;



static FIDESSA_URL: &'static str = "http://data.fidessa.com/CorporateWeb/PriceSummary";


fn push_price(st: &String) {
    use pushover::SyncAPI;
    use pushover::requests::message::SendMessage;
    let api = SyncAPI::new().expect("API fail");
    let msg = SendMessage::new(
        "apxs8vi8bgdechv3gp962svy8b8fod",
        "u7tx6dxkdjwxrkgtq6xi3erxsuyvw1",
        st.clone(),
    );
    let response = api.send(&msg);
    println!("{:?}", response);

}


fn operate_window() -> bool {
    use chrono::prelude::*;

    //    let l: DateTime<Local> = Local::now();
    let l = Local::now();

    let wday = l.weekday().number_from_monday();

    println!("Day of week {} time {:?}", wday, l);

    let minutes = std::time::Duration::from_secs(60 * 15);
    std::thread::park_timeout(minutes);

    wday < 6
}


fn get_share_price() -> Option<String> {
    use curl::easy::Easy;
    let mut handle = Easy::new();

    let mut optional = None;

    handle.url(FIDESSA_URL).unwrap();

    let mut data = Vec::new();
    {
        let mut transfer = handle.transfer();
        transfer
            .write_function(|new_data| {
                data.extend_from_slice(new_data);
                Ok(new_data.len())
            })
            .unwrap();
        transfer.perform().unwrap();
    }

    let mut st = String::from_utf8(data).unwrap();
    let searchst = "Last Price</td>\n        <td class=\'data\'>";

    let x = st.find(searchst).unwrap_or(0);

    if x > 0 {
        let y = x + searchst.len();
        st.drain(..y);

        let z = st.find("</td>").unwrap_or(0);
        if z > 0 {
            st.truncate(z);
            println!("{}", st);
            optional = Some(st);
        }

    }
    optional
}

pub fn price_deviates(mut price: String, lastp: &mut f64) -> bool {
    use std::str::FromStr;

    let optcomma = price.find(',');
    if optcomma.is_some() {
        price.remove(optcomma.unwrap());
    }

    let p: f64 = f64::from_str(&price).unwrap_or(0.0);

    if p == 0.0 {
        println!("Invalid price {}", price);
        return false;
    }

    if *lastp == 0.0 {
        *lastp = p;
    } else {
        let mut diff = p - *lastp;
        if diff < 0.0 {
            diff = -diff;
        }
        if diff > 100.0 {
            *lastp = p;
            return true;
        }
    }
    false
}

fn main() {
    let mut lastp: f64 = 0.0;
    loop {
        if operate_window() {
            let optprice = get_share_price();
            if optprice.is_some() {
                let price = optprice.unwrap();
                if price_deviates(price.clone(), &mut lastp) {
                    let displaymessage = format!("{} {:?}", price, FIDESSA_URL);
                    push_price(&displaymessage);
                }
            }
        }
    }
}
