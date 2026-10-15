//! Project-wide windows and private read cursors use a disposable schema.
use horbyta::{dtos::scope::*, migrations, services::project_conversation as chat};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DbBackend, Statement, TransactionTrait};
use serde_json::json;
use uuid::Uuid;
#[tokio::test]
#[ignore = "Requires PostgreSQL; isolated schema only"]
async fn project_windows_permissions_and_reads() {
    dotenvy::dotenv().ok();
    let url = std::env::var("SCOPE_TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap();
    let mut options = ConnectOptions::new(url);
    options.max_connections(1).min_connections(1).sqlx_logging(false);
    let db = Database::connect(options).await.unwrap();
    let schema = format!("conversation_test_{}", Uuid::now_v7().simple());
    db.execute_unprepared(&format!("CREATE SCHEMA {schema}; SET search_path TO {schema},public"))
        .await
        .unwrap();
    // Cleanup runs even when the test assertion panics.
    use futures::FutureExt;
    let result=std::panic::AssertUnwindSafe(async {
 migrations::migrate(&db).await.unwrap();
 let owner=Uuid::now_v7();let client=Uuid::now_v7();let actor=Uuid::now_v7();let project=Uuid::now_v7();let other=Uuid::now_v7();let hidden=Uuid::now_v7();
 db.execute_unprepared(&format!("INSERT INTO users(id,name,email,password_hash,role) VALUES('{owner}','Owner','conversation@horbyta.test','hash','admin'); INSERT INTO clients(id,user_id,name) VALUES('{client}','{owner}','Client'),('{other}','{owner}','Other'); INSERT INTO client_auth(id,user_id,client_id,email,password_hash) VALUES('{actor}','{owner}','{client}','client@horbyta.test','hash'); INSERT INTO projects(id,user_id,client_id,name) VALUES('{project}','{owner}','{client}','Project'),('{hidden}','{owner}','{other}','Hidden project')")).await.unwrap();
 let dto=|text:&str|ProjectMessageDto{text:text.into(),attachment_ids:vec![],scope_revision:None,change_request_id:None,document_id:None,mentions:vec![]};
 let first=chat::send(&db,owner,None,owner,project,dto("Before any scope publication")).await.unwrap();
 assert!(chat::send(&db,owner,Some(client),actor,hidden,dto("Unauthorized")).await.is_err());
 let mut invalid=dto("Private reference");invalid.mentions.push(ConversationMentionDto{kind:"project".into(),id:hidden,start:None,end:None});
 assert!(chat::send(&db,owner,None,owner,project,invalid).await.is_err());
 let related=Uuid::now_v7();let orphan=Uuid::now_v7();let other_orphan=Uuid::now_v7();
 db.execute_unprepared(&format!("INSERT INTO projects(id,user_id,client_id,name) VALUES('{related}','{owner}','{client}','Related'); INSERT INTO projects(id,user_id,name) VALUES('{orphan}','{owner}','Orphan'),('{other_orphan}','{owner}','Other orphan')")).await.unwrap();
 assert_eq!(chat::resources(&db,owner,None,orphan,"").await.unwrap().as_array().unwrap().len(),1);
 assert!(chat::resources(&db,owner,Some(client),project,"").await.unwrap().as_array().unwrap().iter().any(|r|r["id"]==related.to_string()));
 let references=chat::resources(&db,owner,None,project,"").await.unwrap();
 assert!(!references.as_array().unwrap().iter().any(|r|r["id"]==hidden.to_string()));
 db.execute_unprepared(&format!("INSERT INTO project_messages(project_id,actor_id,actor_kind,text) SELECT '{project}','{owner}','professional','Message '||i FROM generate_series(1,250) i")).await.unwrap();
 let newest=chat::window(&db,owner,Some(client),project,Default::default()).await.unwrap();
 assert_eq!(newest["items"].as_array().unwrap().len(),100);assert_eq!(newest["has_older"],true);assert_eq!(newest["has_newer"],false);
 let cursor=Uuid::parse_str(newest["items"][0]["id"].as_str().unwrap()).unwrap();
 let older=chat::window(&db,owner,Some(client),project,ConversationWindowQuery{before:Some(cursor),..Default::default()}).await.unwrap();
 assert_eq!(older["items"].as_array().unwrap().len(),100);assert_eq!(older["has_newer"],true);
 assert_eq!(chat::inbox(&db,owner,Some(client),actor).await.unwrap()[0]["unread_count"],251);
 chat::read(&db,owner,Some(client),actor,project,first).await.unwrap();
 assert_eq!(chat::inbox(&db,owner,Some(client),actor).await.unwrap()[0]["unread_count"],250);
 let last=Uuid::parse_str(newest["items"][99]["id"].as_str().unwrap()).unwrap();
 chat::read(&db,owner,Some(client),actor,project,last).await.unwrap();
 assert_eq!(chat::inbox(&db,owner,Some(client),actor).await.unwrap(),json!([]));
 chat::read(&db,owner,Some(client),actor,project,first).await.unwrap();
 assert_eq!(chat::inbox(&db,owner,Some(client),actor).await.unwrap(),json!([]));
 let foreign=chat::send(&db,owner,None,owner,hidden,dto("Hidden")).await.unwrap();
 assert!(chat::read(&db,owner,Some(client),actor,project,foreign).await.is_err());
 let around=chat::window(&db,owner,Some(client),project,ConversationWindowQuery{around:Some(first),..Default::default()}).await.unwrap();
 assert!(around["items"].as_array().unwrap().len()<=100);assert_eq!(around["focus_id"],first.to_string());

 let scope_document=||ScopeDocument{deliverables:"Site".into(),criteria:"Working navigation".into(),restrictions:"One delivery".into(),exclusions:"No shop".into(),requirements:"Accessible".into(),..Default::default()};
 horbyta::services::scope::ScopeService::publish(&db,owner,project,PublishScopeDto{expected_revision:0,publish:true,document:scope_document()}).await.unwrap();
 horbyta::services::scope::ScopeService::feedback(&db,owner,Some(client),actor,project,ScopeFeedbackDto{revision:1,kind:"reject".into(),text:"Please revise".into(),document_id:None,attachment_ids:vec![]}).await.unwrap();
 horbyta::services::scope::ScopeService::publish(&db,owner,project,PublishScopeDto{expected_revision:1,publish:true,document:scope_document()}).await.unwrap();
 horbyta::services::scope::ScopeService::feedback(&db,owner,None,owner,project,ScopeFeedbackDto{revision:1,kind:"comment".into(),text:"Legacy revision 1".into(),document_id:None,attachment_ids:vec![]}).await.unwrap();
 let current=chat::send(&db,owner,None,owner,project,dto("Current revision 2")).await.unwrap();
 horbyta::services::scope::ScopeService::accept(&db,owner,client,actor,project,2).await.unwrap();
 let timeline=chat::window(&db,owner,Some(client),project,Default::default()).await.unwrap();
 let entries=timeline["items"].as_array().unwrap();
 assert_eq!(entries.len(),100);
 assert!(entries.iter().any(|e|e["text"]=="Legacy revision 1"));
 assert!(entries.iter().any(|e|e["id"]==current.to_string() && e["context"]["scope_revision"]==2));
 assert!(entries.iter().any(|e|e["kind"]=="scope_rejected"));
 assert!(entries.iter().any(|e|e["kind"]=="scope_accepted"));
 assert_eq!(entries.iter().filter(|e|e["kind"]=="scope_published").count(),2);
 let draft=horbyta::services::change_requests::ChangeRequestService::create(&db,owner,horbyta::dtos::change_request::CreateChangeRequestDto{project_id:project,client_id:client,task_id:None,title:"Private draft".into(),client_request_description:"Not published".into()}).await.unwrap();
 assert!(horbyta::services::scope::ScopeService::change(&db,owner,Some(client),draft.id).await.is_err());
 assert!(!chat::resources(&db,owner,None,project,"").await.unwrap().as_array().unwrap().iter().any(|r|r["id"]==draft.id.to_string()));
 let markdown=Uuid::now_v7(); let audio=Uuid::now_v7();
 db.execute_unprepared(&format!("INSERT INTO attachments(id,user_id,project_id,original_name,storage_key,mime_type) VALUES('{markdown}','{owner}','{project}','criteria.md','conversation/criteria.md','text/markdown'),('{audio}','{owner}','{project}','voice.mp3','conversation/voice.mp3','audio/mpeg')")).await.unwrap();
 assert!(chat::file_resources(&db,owner,None,project,"").await.unwrap().as_array().unwrap().iter().any(|r|r["id"]==markdown.to_string()));
 assert!(chat::file_resources(&db,owner,Some(client),project,"").await.unwrap().as_array().unwrap().is_empty());
 let mut file_only=dto("");file_only.attachment_ids.push(markdown);
 let file_message=chat::send(&db,owner,Some(client),actor,project,file_only).await.unwrap();
 assert!(chat::file_resources(&db,owner,Some(client),project,"").await.unwrap().as_array().unwrap().iter().any(|r|r["id"]==markdown.to_string()));
 let mut file_reference=dto("#criteria.md check");
 file_reference.mentions.push(ConversationMentionDto{kind:"file".into(),id:markdown,start:Some(0),end:Some(12)});
 let referenced=chat::send(&db,owner,Some(client),actor,project,file_reference).await.unwrap();
 let referenced_window=chat::window(&db,owner,None,project,Default::default()).await.unwrap();
 assert!(referenced_window["items"].as_array().unwrap().iter().any(|e|e["id"]==referenced.to_string() && e["mentions"][0]["mime_type"]=="text/markdown"));
 let with_file=chat::window(&db,owner,None,project,Default::default()).await.unwrap();
 assert!(with_file["items"].as_array().unwrap().iter().any(|e|e["id"]==file_message.to_string() && e["attachments"][0]["original_name"]=="criteria.md"));
 let mut unsupported=dto("");unsupported.attachment_ids.push(audio);
 let tx=db.begin().await.unwrap();
 assert!(chat::send(&tx,owner,Some(client),actor,project,unsupported).await.is_err());
 tx.rollback().await.unwrap();
 assert!(db.execute(Statement::from_sql_and_values(DbBackend::Postgres,"DELETE FROM attachments WHERE id=$1",[markdown.into()])).await.is_err());
 let mut inline=dto("😀 @Project please");
 inline.mentions.push(ConversationMentionDto{kind:"project".into(),id:project,start:Some(2),end:Some(10)});
 let inline_id=chat::send(&db,owner,Some(client),actor,project,inline).await.unwrap();
 let inline_window=chat::window(&db,owner,None,project,Default::default()).await.unwrap();
 assert!(inline_window["items"].as_array().unwrap().iter().any(|e|e["id"]==inline_id.to_string() && e["mentions"][0]["start"]==2 && e["mentions"][0]["label"]=="Project"));
 let mut invalid_span=dto("Unrelated text");
 invalid_span.mentions.push(ConversationMentionDto{kind:"project".into(),id:project,start:Some(0),end:Some(8)});
 let tx=db.begin().await.unwrap();
 assert!(chat::send(&tx,owner,Some(client),actor,project,invalid_span).await.is_err());
 tx.rollback().await.unwrap();
 // Stored conversation is immutable, independently of the UI.
 assert!(db.execute(Statement::from_sql_and_values(DbBackend::Postgres,"UPDATE project_messages SET text='Changed' WHERE id=$1",[first.into()])).await.is_err());
 }).catch_unwind().await;
    db.execute_unprepared(&format!("SET search_path TO public; DROP SCHEMA {schema} CASCADE"))
        .await
        .unwrap();
    if let Err(error) = result {
        std::panic::resume_unwind(error);
    }
}
