use actix_web::{web, HttpResponse};

use crate::authentication::UserId;

pub async fn get_newsletter_page(
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let _user_id = user_id.into_inner();
    let html = r#"
    <html>
        <head>
            <title>Admin Newsletter</title>
        </head>
        <body>
            <h1>Send Newsletter</h1>
            <form action="newsletter" method="post">
                <input type="text" name="title" placeholder="Title" required>
                <br />
                <textarea name="html_content" placeholder="html content" required></textarea>
                <textarea name="content" placeholder="content" required></textarea>
                <br /> 
                <button type="submit">Send Issue</button>
            </form>
        </body>
    </html>
    "#;

    Ok(HttpResponse::Ok().body(html))
}
