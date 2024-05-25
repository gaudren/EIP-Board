use std::collections::HashSet;

use askama::Template;
use chrono::{DateTime, Utc};
use eipw_preamble::Preamble;
use octocrab::models::pulls::ReviewState;
use octocrab::params;
use octocrab::{models::pulls::PullRequest, Octocrab};
use regex::{Regex, RegexSet};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    urls: Vec<String>,
}

#[tokio::main]
async fn main() -> octocrab::Result<()> {
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env variable is required");
    let repo =
        std::env::var("GITHUB_REPOSITORY").expect("GITHUB_REPOSITORY env variable is requried");

    let (owner, repo) = repo
        .split_once("/")
        .expect("No slash in GitHub repository.");

    let octocrab = Octocrab::builder().personal_token(token).build()?;
    let editors = editors(&octocrab, owner, repo).await?;
    let opr = open_pull_requests(&octocrab, owner, repo).await?;

    let mut needs_review = Vec::new();

    for pr in opr {
        let reviewed_at = match reviewed_by_editor(&octocrab, &editors, &pr, owner, repo).await? {
            None => {
                needs_review.push((pr.created_at, pr.html_url));
                continue;
            }
            Some(r) => r,
        };

        let updated_at = match pr.updated_at {
            Some(u) => u,
            None => continue,
        };

        authors(&octocrab, &pr, owner, repo).await?;

        if updated_at > reviewed_at {
            needs_review.push((pr.created_at, pr.html_url));
        }
    }

    needs_review.sort();

    let index = IndexTemplate {
        urls: needs_review
            .into_iter()
            .filter_map(|x| x.1)
            .map(|x| x.to_string())
            .collect(),
    };
    println!("{}", index.render().unwrap());
    Ok(())
}

async fn reviewed_by_editor(
    oct: &Octocrab,
    editors: &HashSet<String>,
    open_pr: &PullRequest,
    owner: &str,
    repo: &str,
) -> octocrab::Result<Option<DateTime<Utc>>> {
    let reviews = oct
        .pulls(owner, repo)
        .list_reviews(open_pr.number)
        .per_page(100)
        .page(1u32)
        .send()
        .await?;

    assert!(matches!(reviews.next, None));

    let reviews = reviews.items;
    if reviews.is_empty() {
        return Ok(None);
    }

    let mut reviewers: Vec<_> = reviews
        .into_iter()
        .filter(|x| {
            matches!(
                x.state,
                Some(ReviewState::ChangesRequested | ReviewState::Commented)
            )
        })
        .filter(|x| {
            let user = match &x.user {
                Some(u) => u,
                None => return false,
            };
            editors.contains(&user.login)
        })
        .collect();

    reviewers.sort_by_key(|x| x.submitted_at);

    match reviewers.last() {
        Some(u) => Ok(u.submitted_at),
        None => Ok(None),
    }
}

async fn open_pull_requests(
    oct: &Octocrab,
    owner: &str,
    repo: &str,
) -> octocrab::Result<Vec<PullRequest>> {
    let mut current_page = oct
        .pulls(owner, repo)
        .list()
        .state(params::State::Open)
        // TODO: Filter out drafts
        .per_page(100)
        .send()
        .await?;

    let mut prs = current_page.take_items();

    while let Some(mut new_page) = oct.get_page(&current_page.next).await? {
        prs.extend(new_page.take_items());

        current_page = new_page;
    }

    Ok(prs)
}

async fn editors(oct: &Octocrab, owner: &str, repo: &str) -> octocrab::Result<HashSet<String>> {
    let mut content = oct
        .repos(owner, repo)
        .get_content()
        .path("config/eip-editors.yml")
        .r#ref("master")
        .send()
        .await?;

    let contents = content.take_items();
    let c = &contents[0];
    let decoded_content = c.decoded_content().unwrap();

    let re = Regex::new(r"(?m)^  - (.+)").unwrap();

    let mut results = HashSet::new();
    for (_, [username]) in re.captures_iter(&decoded_content).map(|c| c.extract()) {
        results.insert(username.to_string());
    }

    Ok(results)
}

async fn authors(
    oct: &Octocrab,
    pr: &PullRequest,
    owner: &str,
    repo: &str,
) -> octocrab::Result<HashSet<String>> {
    let mut files = oct.pulls(owner, repo).list_files(pr.number).await?;
    assert!(files.next.is_none());
    let re = Regex::new(r"(EIPS|ERCS)/(eip|erc)-[0-9]+\.md").unwrap();
    let files = files.take_items().into_iter().filter_map(|x| {
        if re.is_match(&x.filename) {
            Some(x.filename)
        } else {
            None
        }
    });

<<<<<<< Updated upstream
    //let mut authors = HashSet::new();
    let repo = pr.head.repo.as_ref().unwrap();

=======
    let mut author_set = HashSet::new();
    let repo = pr.head.repo.as_ref().unwrap();

    let re = Regex::new(
            r"^[^()<>,@]+ \(@([a-zA-Z\d-]+)\)(?: <[^@][^>]*@[^>]+\.[^>]+>)?$")
        .unwrap();

>>>>>>> Stashed changes
    for file in files {
        let mut content = oct
            .repos(&repo.owner.as_ref().unwrap().login, &repo.name)
            .get_content()
<<<<<<< Updated upstream
            .path(file)
=======
            .path(&file)
>>>>>>> Stashed changes
            .r#ref(&pr.head.ref_field)
            .send()
            .await?;
        let contents = content.take_items();
        let c = &contents[0];
        let decoded_content = c.decoded_content().unwrap();

<<<<<<< Updated upstream
        println!("{decoded_content}")
    }
    todo!()
=======
        let (preamble, _) = Preamble::split(&decoded_content).unwrap();
        let preamble = Preamble::parse(Some(&file), preamble).unwrap();
        let authors = preamble.by_name("author").unwrap().value().trim();

        let authors = authors.split(',').map(str::trim);

        for author in authors {
            let captures = match re.captures(author) {
                Some(s) => s,
                None => continue,
            };
            let username = captures.get(1).unwrap().as_str().to_lowercase();
            author_set.insert(username);
        }
    }
    println!("{:#?}", author_set);
    Ok(author_set)
>>>>>>> Stashed changes
}
