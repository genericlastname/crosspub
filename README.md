# crosspub

A tool for users on tilde servers to cross publish blog posts to HTML and
Gemini.

## Installation

### Cargo
```
cargo install crosspub
```

### Binary
Linux binaries are available on the [Releases page](https://github.com/genericlastname/crosspub/releases)

### From source
Run
```
make
sudo make install
```

## Usage

crosspub is designed to not need any configuration out of the box. To start
create a containing directory and initialize it.

```
mkdir website
crosspub --init
```

This creates two directories, one for `posts` (blogs and articles written on a
specific day) and one for `topics` (evolving documents that change and grow over
time), as well as creating a default config file.

crosspub content is written in
[gemtext](https://gemini.circumlunar.space/docs/gemtext.gmi), a lightweight
markup language. Files should end in `.gmi`.

Next, change the settings in your config.toml, see [Basic
Configuration](#Basic-Configuration).

Create posts directories in your HTML and Gemini root directories
```
mkdir ~/public_html/posts
mkdir ~/public_gemini/posts
```

Change these paths to match your configuration.

When you're ready to generate your website just navigate to the directory you
initialized and run

```
crosspub
```

### Posts syntax

All gemtext files in posts/ must start with a mandatory TOML frontmatter

```
---
title = "Example Title"
date = "YYYY-MM-DD"
slug = "example"
---

Content goes here
...
```

The "slug" is a small string that becomes part of the filename, basically a
shortened title.

### Topics syntax

Files in topics/ start with a slightly different frontmatter that lacks a date
field.

```
---
title = "Example Topic"
slug = "example_topic"
---

...
```

## Basic Configuration

The config file is located at `$HOME/crosspub/config.toml`

- `name` is the title of your site
- `url` is the base URL of the site (not including the tilde part)
- `username` is your username, your tilde extension on the site minus the `~`
character
- `html_root` and `gemini_root` are the paths to your public\_html and
public\_gemini files.

## About files

crosspub allows you to write a bio and have it generate an About page. First
make sure you have a directory for crosspub data:

```
mkdir -p ~/.local/share/crosspub
```

Next create a file in that directory called `about.gmi`. This is just a plain
gemtext file, no frontmatter.

Finally edit `~/.config/crosspub/config.toml` so that `use_about_page = true`

## Advanced Configuration

### Custom Templates

If desired, more unique sites can be created by utilizing custom templates.
First create the proper directories

```
mkdir -p ~/.local/share/crosspub/templates/html
mkdir -p ~/.local/share/crosspub/templates/gemini
```

The default templates are located at `/usr/share/crosspub/templates/`. It is
recommended to look at the built-in templates for an explanation of how they
work.

crosspub uses 5 templates each for HTML and Gemini
- index
- post
- topic
- postlist
- about

Some or all of these templates can be shadowed by ones located in
`~/.local/share/crosspub/templates`.

### Custom CSS

Similar to the templates, site-wide CSS can be modified. User CSS should go in
`~/.local/share/crosspub/templates/html/style.css`

### Post Listing

The default index.html and index.gmi templates both list posts on the homepage.
If you'd like to move this listing to a separate page set `post_list = true` in
your config. This creates a listing at `{HTML_ROOT}/posts/posts.html` and
`{GEMINI_ROOT}/posts/posts.gmi`. crosspub will NOT automatically link to these
listings, so it's up to you to modify other templates as necessary.
