pub mod crosspub;
pub mod gemtext;
pub mod post;
pub mod settings;

use crosspub::CrossPub;

fn main() {
    // let p = std::path::Path::new("/home/hiroantag/projects/writings/test.gmi");
    // let mut post_list = Vec::new();
    let config = settings::Config {
        name: String::from("hiroantag's web living room"),
        url: String::from("retrace.club/"),
        username: String::from("hiroantag"),
        default_index: true,
        about: String::from(""),
        html_template: String::from("templates/post.html"),
        gemini_template: String::from(""),
        default_style: true,
        css: String::from(""),
        html_root: String::from("/home/hiroantag/public_html"),
        gemini_root: String::from("/home/hiroantag/public_gemini"),
        post_dir: String::from("posts"),
    };
    let mut cp = CrossPub::new(&config);
    cp.load_dir("test_gmi");
    cp.write_html_posts(true);
}
