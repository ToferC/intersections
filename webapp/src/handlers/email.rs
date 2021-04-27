
use std::env;

use actix_web::{HttpRequest, HttpResponse, Responder, post, web};
use actix_identity::{Identity};
use serde::{Deserialize};
use tera::Context;
use regex::Regex;

use crate::{AppData};
use crate::models::{Communities, CommunityData, Email, People};

// for for single email address
#[derive(Debug, Deserialize)]
pub struct EmailForm {
    pub email: String,
}
// form for multiple email addresses
#[derive(Debug, Deserialize)]
pub struct EmailsForm {
    pub emails: String,
}

#[post("/{lang}/person/{code}")]
pub async fn email_person_info(
    web::Path((lang, code)): web::Path<(String, String)>,
    data: web::Data<AppData>,
    _req: HttpRequest,
    _id: Identity,
    form: web::Form<EmailForm>,
) -> impl Responder {

    // validate form has data or re-load form
    if form.email.is_empty() {
        return HttpResponse::Found().header("Location", format!("/{}/person/{}", &lang, code)).finish()
    };

    let person = People::find_from_code(&code);

    match person {
        Ok(person) => {
            let mut ctx = Context::new();

            let community = Communities::find(person.community_id).unwrap();

            let application_url: String;
            let environment = env::var("ENVIRONMENT").unwrap();

            if environment == "production" {
                application_url = format!("https://intersectional-data.ca/{}", &lang);
            } else {
                application_url = format!("http://localhost:8088/{}", &lang);
            };

            // add qr code to add profile (prod only)
            // let qr = qrcode_generator::to_svg_to_string(format!("{}/{}", application_url, &person.code), QrCodeEcc::Low, 245, Some("Invitation link for intersections")).unwrap();
            // ctx.insert("qrcode", &qr);


            ctx.insert("person", &person);
            ctx.insert("community", &community);
            ctx.insert("application_url", &application_url);

            let rendered = data.tmpl.render("emails/email_person.html", &ctx).unwrap();
            
            let email = Email::new(
                form.email.to_owned(),
                rendered,
                String::from("Your personal data link from Intersectional-Data.ca"), 
                data.mail_client.clone(),
            );

            let r = Email::send(&email).await;

            match r {
                Ok(_) => println!("Message sent"),
                _ => println!("Message not sent"),
            };

            return HttpResponse::Found().header("Location", format!("/{}/person/{}", &lang, code)).finish()
        },
        Err(err) => {
            println!("Error: {}", err);
            return HttpResponse::Found().header("Location", format!("/{}/person/{}", &lang, code)).finish()
        }
    };
}

#[post("/send_community_email/{slug}")]
pub async fn send_community_email(
    web::Path((lang, slug)): web::Path<(String, String)>,
    data: web::Data<AppData>,
    _req: HttpRequest,
    _id: Identity,
    form: web::Form<EmailsForm>,
) -> impl Responder {

    // instantiate regex
    let re = Regex::new(r"([a-zA-Z0-9+._-]+@[a-zA-Z0-9._-]+\.[a-zA-Z0-9_-]+)").unwrap();
    
    let mut emails: Vec<String> = Vec::new();
    
    for mat in re.captures_iter(form.emails.as_str()) {
        emails.push(mat[0].to_owned());
    };
    
    // validate form had emails or re-load page
    if emails.is_empty() {
        return HttpResponse::Found().header("Location", format!("/{}/community/{}", &lang, slug)).finish()
    };

    let community = Communities::find_from_slug(&slug);

    match community {
        Ok(mut community) => {
            let mut ctx = Context::new();

            let mut comm_data: CommunityData = serde_json::from_value(community.data.to_owned()).expect("Unable to retrieve community data");

            let application_url: String;
            let environment = env::var("ENVIRONMENT").unwrap();

            if environment == "production" {
                application_url = format!("https://intersectional-data.ca/{}", &lang);
            } else {
                application_url = format!("http://localhost:8088/{}", &lang);
            };

            let community_add_profile_url = format!("{}/survey_intro/{}", application_url, &community.code);
            ctx.insert("add_community_profile_url", &community_add_profile_url);

            ctx.insert("community", &community);

            let rendered = data.tmpl.render("emails/email_community_invitation.html", &ctx).unwrap();

            // Send emails
            for email in &emails {

                println!("{}", &email);

                let e = Email::new(
                    email.to_owned(),
                    rendered.clone(),
                    String::from("Your personal data link from Intersectional-Data.ca"), 
                    data.mail_client.clone(),
                );
    
                let r = Email::send(&e).await;
    
                match r {
                    Ok(_) => {
                        println!("Message to {} sent", &email);
                        comm_data.invitations += 1;
                    },
                    _ => println!("Message to {} not sent", &email),
                };
            };

            
            // Send email to community owner for reference
            let owner_email = Email::new(
                community.contact_email.to_owned(), 
                format!("Email invitations sent to: {:?}<br>
                {} of {} invitations sent successfully", 
                &emails,
                &comm_data.invitations,
                &emails.len()),
                format!("Reference: invitations sent for {} community on intersectional-data.ca", &community.tag), 
                data.mail_client.clone()
            );
            
            let r = Email::send(&&owner_email).await;
            
            match r {
                Ok(_) => println!("Message to owner sent"),
                _ => println!("Message to owner not sent"),
            };

            // Update community invitations
            community.data = serde_json::to_value(comm_data).expect("Unable to serialize comm data");
            Communities::update(&community).expect("Unable to update community with invitations");

            return HttpResponse::Found().header("Location", format!("/{}/community/{}", &lang, slug)).finish()
        },
        Err(err) => {
            println!("Error: {}", err);
            return HttpResponse::Found().header("Location", format!("/{}/community/{}", &lang, slug)).finish()
        }
    };
}