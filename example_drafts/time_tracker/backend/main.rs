use moon::*;
use shared::{UpMsg, DownMsg, Message, ClientId, ProjectId};

mod client;
mod invoice;
mod project;
mod time_block;
mod time_entry;

async fn request_handler(req: Request) {
    let down_msg = match req.up_msg {

        // ------ Page data ------
        UpMsg::GetClientsAndProjectsClients => {
            let shared_clients_futs = client::by_id().iter().map(|(id, client)| {
                async {
                    let (name, projects) = join(client.name(), client.projects()).await;

                    let shared_projects_futs = projects.iter().map(|(id, project)| {
                        async {
                            shared::clients_and_projects::Project {
                                id,
                                name: project.name().await
                            }
                        }
                    });

                    shared::clients_and_projects::Client {
                        id,
                        name,
                        projects: join_all(shared_projects_futs).await 
                    }
                }
            });
            DownMsg::ClientsAndProjectsClients(
                join_all(shared_clients_futs).await
            )
        }
        UpMsg::TimeBlocksClients => {
            let shared_clients_futs = client::by_id().iter().map(|(id, client)| {
                async {
                    let (name, tracked, time_blocks) = join!(
                        client.name(), client.tracked(), client.time_blocks()
                    ).await;

                    let shared_time_blocks_futs = time_blocks.iter().map(|(id, time_block)| {
                        async {
                            let (name, status, duration, invoice) = join!(
                                time_block.name(), time_block.status(), time_block.duration(), time_block.invoice()
                            ).await;

                            let invoice = if let Some((id, invoice)) = invoice {
                                let (custom_id, url) = join!(invoice.custom_id, invoice.url).await;
                                Some(shared::time_blocks::Invoice { id, custom_id, url })
                            } else {
                                None
                            };

                            shared::time_blocks::TimeBlock { id, name, status, duration, invoice }
                        }
                    });

                    shared::time_blocks::Client {
                        id,
                        name,
                        time_blocks: join_all(shared_time_blocks_futs).await,
                        tracked, 
                    }
                }
            });
            DownMsg::ClientsAndProjectsClients(
                join_all(shared_clients_futs).await
            )
        }
        UpMsg::TimeTrackerClients => {
            let shared_clients_futs = client::by_id().iter().map(|(id, client)| {
                async {
                    let (name, projects) = join(client.name(), client.projects()).await;

                    let shared_projects_futs = projects.iter().map(|(id, project)| {
                        async {
                            let (name, time_entries) = join(project.name, project.time_entries).await;
                            
                            let shared_time_entries_futs = time_entries.iter().map(|(id, time_entry)| {
                                async {
                                    let (name, started, stopped) = join!(
                                        time_entry.name, time_entry.started, time_entry.stopped
                                    );
                                    shared::time_tracker::TimeEntry { id, name, started, stopped }
                                }
                            });
                            
                            shared::time_tracker::Project {
                                id,
                                name,
                                time_entries: join_all(shared_time_entries_futs).await,
                            }
                        }
                    });

                    shared::time_tracker::Client {
                        id,
                        name,
                        projects: join_all(shared_projects_futs).await 
                    }
                }
            });
            DownMsg::ClientsAndProjectsClients(
                join_all(shared_clients_futs).await
            )
        }

        // ------ Client ------
        UpMsg::AddClient(id) => {
            new_actor(Client { id }).await;
            DownMsg::ClientAdded
        },
        UpMsg::RemoveClient(id) => {
            client::by_id().get(id)[0].remove().await;
            DownMsg::ClientRemoved
        },
        UpMsg::RenameClient(id, name) => {
            client::by_id().get(id)[0].rename(name.to_string()).await;
            DownMsg::ClientRenamed
        },

        // ------ Project ------
        UpMsg::AddProject(client, id) => {
            new_actor(Project { client, id }).await;
            DownMsg::ProjectAdded
        },
        UpMsg::RemoveProject(id) => {
            project::by_id().get(id)[0].remove().await;
            DownMsg::ProjectRemoved
        },
        UpMsg::RenameProject(id, name) => {
            project::by_id().get(id)[0].rename(name.to_string()).await;
            DownMsg::ProjectRenamed
        },

        // ------ TimeBlock ------
        UpMsg::AddTimeBlock(client, id, duration) => {
            new_actor(TimeBlock { client, id, duration }).await;
            DownMsg::TimeBlockAdded
        },
        UpMsg::RemoveTimeBlock(id) => {
            time_block::by_id().get(id)[0].remove().await;
            DownMsg::TimeBlockRemoved
        },
        UpMsg::RenameTimeBlock(id, name) => {
            time_block::by_id().get(id)[0].rename(name.to_string()).await;
            DownMsg::TimeBlockRenamed
        },
        UpMsg::SetTimeBlockStatus(id, status) => {
            time_block::by_id().get(id)[0].set_status(status).await;
            DownMsg::TimeBlockStatusSet
        },
        UpMsg::SetTimeBlockDuration(id, duration) => {
            time_block::by_id().get(id)[0].set_duration(duration).await;
            DownMsg::TimeBlockDurationSet
        },

        // ------ Invoice ------
        UpMsg::AddInvoice(time_block, id) => {
            new_actor(Invoice { time_block, id }).await;
            DownMsg::InvoiceAdded
        },
        UpMsg::RemoveInvoice(id) => {
            invoice::by_id().get(id)[0].remove().await;
            DownMsg::InvoiceRemoved
        },
        UpMsg::SetInvoiceCustomId(id, custom_id) => {
            invoice::by_id().get(id)[0].set_custom_id(custom_id.to_string()).await;
            DownMsg::InvoiceCustomIdSet
        },
        UpMsg::SetInvoiceUrl(id, url) => {
            invoice::by_id().get(id)[0].set_url(url.to_string()).await;
            DownMsg::InvoiceUrlSet
        },

        // ------ TimeEntry ------
        UpMsg::AddTimeEntry(project, time_entry) => {
            new_actor(TimeEntry { project, time_entry }).await;
            DownMsg::TimeEntryAdded
        },
        UpMsg::RemoveTimeEntry(id) => {
            time_entry::by_id().get(id)[0].remove().await;
            DownMsg::TimeEntryRemoved
        },
        UpMsg::RenameTimeEntry(id, name) => {
            time_entry::by_id().get(id)[0].rename(name.to_string()).await;
            DownMsg::TimeEntryRenamed
        },
        UpMsg::SetTimeEntryStarted(id, started) => {
            time_entry::by_id().get(id)[0].set_started(started).await;
            DownMsg::TimeEntryStartedSet
        },
        UpMsg::SetTimeEntryStopped(id, stopped) => {
            time_entry::by_id().get(id)[0].set_stopped(stopped).await;
            DownMsg::TimeEntryStoppedSet
        },
    };
    connected_client::by_id().get(req.client_id)[0].send_down_msg(down_msg).await
}

fn main() {
    start!(request_handler, actors![client, invoice, project, time_block, time_entry]);
}
