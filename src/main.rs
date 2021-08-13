mod utils;
mod hacker_news;
use std::error::Error;
use std::{env, time::Duration, time::UNIX_EPOCH};
use std::{io, vec};
use chrono::DateTime;
use chrono::Utc;
use tui::text::Text;
use tui::{
    Terminal, 
    backend::TermionBackend, 
    layout::{Layout, Constraint, Direction}, 
    text::{Spans, Span}, 
    widgets::{Block, Borders, List, ListItem}, 
    style::{Color, Modifier, Style}
};

use sanitize_html::{
    sanitize_str,
    rules::predefined::DEFAULT
};

use crate::hacker_news::{
    HNPost,
    HNComment,
    get_data,
    get_comments
};

use crate::utils::{
    StatefulList,
    events::{Events, Event}
};

use termion::{raw::IntoRawMode, event::Key};

struct HNPostsList {
    items: StatefulList<HNPost>,
    selected_post_comments: Vec<HNComment>
}

fn main() ->  Result<(), Box<dyn Error>>{
    let args: Vec<String> = env::args().collect();
    let category = &args[1];
    let mut selected_post_index = 0;
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let hn_posts: Vec<HNPost> = get_data(category).unwrap();
    let events = Events::new();
    let mut hn_posts_list = HNPostsList{
        items: StatefulList::with_items(hn_posts),
        selected_post_comments: vec![]
    };
    terminal.clear()?;
    loop {
        terminal.draw(|f| {

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(38), Constraint::Percentage(2)].as_ref())
                .split(f.size());

            let top_chunks = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage(70), Constraint::Percentage(30)]).split(chunks[0]);

            let right_top_chunks = Layout::default().direction(Direction::Vertical).constraints([Constraint::Percentage(30), Constraint::Percentage(70)]).split(top_chunks[1]);

            let posts: Vec<ListItem> = hn_posts_list.items.items.iter().map(|post| {
                let spans: Vec<Spans> = vec![Spans::from(vec![
                    Span::raw(post.title.to_string())
                ])];
                ListItem::new(spans)
            }).collect();

            let post_details: Vec<List> = hn_posts_list.items.items.iter().map(|post| {
                let by_span: Vec<Spans> = vec![Spans::from(
                    vec![
                        Span::raw(format!("{} {}", "By", post.by ))
                    ]
                )];

                let time_span: Vec<Spans> = vec![Spans::from(
                    vec![
                        Span::raw(format!("{} {}", "Posted on", DateTime::<Utc>::from(UNIX_EPOCH + Duration::from_secs(post.time)).format("%d-%m-%Y %H:%M:%S"))),
                    ]
                )];

                let score_span: Vec<Spans> = vec![Spans::from(
                    vec![
                        Span::raw(format!("{} {}", "Score", post.score ))
                    ]
                )];

                let url = if post.url.is_some() {
                    post.url.clone().unwrap()
                } else {
                    format!("https://news.ycombinator.com/item?id={}",post.id)
                };

                let url_span: Vec<Spans> = vec![Spans::from(
                    vec![
                        Span::raw(format!("{} {}", "Url", url))
                    ]
                )];

                let total_comments = if post.descendants.is_some() {
                    post.descendants.clone().unwrap()
                } else {
                    0
                };

                let total_comments_span: Vec<Spans> = vec![Spans::from(
                    vec![
                        Span::raw(format!("{} {}", "Total Comments", total_comments))
                    ]
                )];

                List::new(vec![ListItem::new(by_span) ,ListItem::new(time_span) ,ListItem::new(score_span), ListItem::new(url_span), ListItem::new(total_comments_span)])
            }).collect();

            let all_posts = List::new(posts)
                    .block(Block::default().borders(Borders::ALL).title("HN Top Posts")) .highlight_style(
                        Style::default()
                            .bg(Color::LightGreen)
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::DarkGray)
                    );
            
            f.render_stateful_widget(all_posts, top_chunks[0], &mut hn_posts_list.items.state);

            let post_info_block = Block::default().borders(Borders::ALL).title("Post Info");

            let selected_post_details = post_details.get(selected_post_index).unwrap();

            let post_details_list = selected_post_details.clone().block(post_info_block);

            f.render_widget(post_details_list, right_top_chunks[0]);

            let blank_block = Block::default();

            f.render_widget(blank_block, right_top_chunks[1]);

            let footer_items: Vec<ListItem> = vec![ListItem::new(vec![
                    Spans::from(
                        vec![
                            Span::raw("(q) to quit\t (->) to open webpage\t (<-) to open hacker news post (c) to load post comments")
                        ]
                    )
            ])];

            let footer = List::new(footer_items).block(Block::default());
                
            f.render_widget(footer, chunks[2]);
            let all_comments: Vec<ListItem> = hn_posts_list.selected_post_comments.clone().into_iter().map(|comment| {
                let unsanitized_comment_value:String;
                let sanitized_comment_value:String;
                let comment_value:String;
                let mut lines: Vec<&str> = vec![];
                if comment.text.is_none() {
                    comment_value = "Dead Commnet".to_string();
                } else {
                    //comment_value = comment.text.unwrap().replace("<p>","/\r/\n").replace("&gt;", ">").replace("&lt;", "<");
                    unsanitized_comment_value = comment.text.unwrap();
                    lines = unsanitized_comment_value.split("<p>").collect();
                    sanitized_comment_value = sanitize_str(&DEFAULT, &lines.join("@@NL@@")).unwrap();
                    lines = sanitized_comment_value.split("@@NL@@").collect();
                    comment_value = lines.join("\n").replace("&lt;", "<").replace("&gt;", ">").replace("&amp;", "&");
                }
                ListItem::new(Text::from(comment_value + "\n----------------------------------------------------------------------------------------------------------------------"))
            }).collect();

            let post_comment_block = List::new(all_comments).block(Block::default().borders(Borders::ALL).title("Post Comments"));

            f.render_widget(post_comment_block, chunks[1]);
        })?;

        match events.next()? {
            Event::Input(input) => match input {
                Key::Char('q') => {
                    break;
                }
                Key::Left => {
                    //hn_posts_list.items.unselect();
                    let selected_index = hn_posts_list.items.state.selected();
                    let selected_post = hn_posts_list.items.items.get(selected_index.unwrap()).unwrap();
                    open::that(format!("https://news.ycombinator.com/item?id={}", selected_post.id))?;
                }
                Key::Down => {
                    hn_posts_list.items.next();
                    selected_post_index = hn_posts_list.items.state.selected().unwrap();
                }
                Key::Up => {
                    hn_posts_list.items.previous();
                    selected_post_index = hn_posts_list.items.state.selected().unwrap();
                }
                Key::Right => {
                    let selected_index = hn_posts_list.items.state.selected();
                    let selected_post = hn_posts_list.items.items.get(selected_index.unwrap()).unwrap();
                    if selected_post.url.is_some() {
                        open::that(selected_post.url.clone().unwrap())?;
                    } else {
                        open::that(format!("https://news.ycombinator.com/item?id={}", selected_post.id))?;
                    }
                },
                Key::Char('c') => {
                    let selected_index = hn_posts_list.items.state.selected();
                    let selected_post = hn_posts_list.items.items.get(selected_index.unwrap()).unwrap();
                    if selected_post.kids.is_some() {
                        hn_posts_list.selected_post_comments =  get_comments(selected_post.kids.clone().unwrap()).unwrap();
                    }
                },
                _ => {}
            },
            Event::Tick => {

            }
        }
    }   
    Ok(())
}
