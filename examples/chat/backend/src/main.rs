use moon::*;
use shared::{UpMsg, DownMsg, Message};

async fn frontend() -> Frontend {
    Frontend::new()
        .title("Chat example")
        .append_to_head("<style>html {
            background-color: black;
            color: lightgray;
        }
        
        #app * {
            padding: 5px;
        }
            
        .button {
            cursor: pointer;
            background-color: darkgreen;
            user-select: none;
        }
        
        .button:hover {
            background-color: green;
        }
        
        .column {
            display: flex;
            flex-direction: column;
        }
        
        .row {
            display: flex;
        }
        </style>")
}

async fn up_msg_handler(_req: UpMsgRequest) {
    println!("UpMsgHandler!");

    // if let UpMsg::SendMessage(message) = req.up_msg {
    //     // join_all(connected_client::by_id().iter().map(|(_, client)| {
    //     //     client.send_down_msg(message, req.cor_id)
    //     // })).await
    // }
}

// #[moon::main]
// async fn main() -> std::io::Result<()> {
//     start(frontend, up_msg_handler, |_|{}).await
// }

fn main() -> std::io::Result<()> {
    actix_web::rt::System::new().block_on(async move { 
        start(frontend, up_msg_handler, |_|{}).await
    })
}


