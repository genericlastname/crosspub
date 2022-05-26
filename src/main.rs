pub mod gemtext;
pub mod html;
pub mod post;
pub mod settings;

fn main() {
    let p = std::path::Path::new("/home/hiroantag/projects/writings/test.gmi");
    let mut post_list = Vec::new();
    let config = settings::Config {
        site_name: String::from("hiroantag's web living room"),
        site_url: String::from("retrace.club/~hiroantag"),
        default_index: true,
        about: String::from(""),
        default_header: true,
        default_footer: true,
        header: String::from(""),
        footer: String::from(""),
        default_style: true,
        css: String::from(""),
        html_root: String::from("/home/hiroantag/public_html"),
        gemini_root: String::from("/home/hiroantag/public_gemini"),
        post_dir: String::from("posts"),
    };
    post::create_post(p, &mut post_list, &config);
}
