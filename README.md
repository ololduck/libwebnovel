# libwebnovel

[![docs.rs](https://img.shields.io/docsrs/libwebnovel)](https://docs.rs/libwebnovel)

This crate deals with webnovels. You can see it as a way to access different
webnovel hosting sites and be able to get their contents.

Since there are times we don't have Internet access, such as when riding
some trains, downloading to disk in a convenient format seems the way to go.

**BY USING THIS CRATE/LIBRARY YOU HEREBY PLEDGE TO NOT PROFIT OF THE
DOWNLOADED FICTIONS IN ANY WAY, OR, BY YOUR ACTION, MAKE AN OTHER ENTITY
PROFIT IN ANY WAY FROM THE DOWNLOADED FICTIONS**. This is serious, this
crate is intended for reading comfort, _not_ to enable people to be
arseholes.

### Example
Say you want to create a software that will generate epubs from a given
fiction url. This could be expressed by something like the following:

```rust
use libwebnovel::{Backend, Backends, Chapter};

fn main() {
    // Get the backend matching the given URL
    let fiction_backend =
        Backends::new("https://www.royalroad.com/fiction/21220/mother-of-learning").unwrap();
    // Get all the chapters of the webnovel
    let chapters = fiction_backend.get_chapters().unwrap();

    // write the resulting epub
    let epub_path = format!("{}.epub", fiction_backend.title().unwrap());
    let mut f = File::create(&epub_path).unwrap();
    write_chapters_to_epub(&mut f, &chapters).unwrap();

    // Since this code example also sort of serves as an integration test,
    // remove the created file :p
    std::fs::remove_file(epub_path).unwrap();
}

fn write_chapters_to_epub(writer: &impl Write, chapters: &[Chapter]) -> Result<(), io::Error> {
    // do stuff to create the ebook here
    Ok(())
}
```

See [`Backends`] for more information on how to use the library. The
documentation of the [`Backend`] trait may also be useful, especially if you
want to implement another backend (don't forget to share it with the [main repository](https://codeberg.org/paulollivier/libwebnovel)!).

### Supported providers

- [RoyalRoad](https://www.royalroad.com/)
- [FreeWebNovel](https://freewebnovel.com/)
- [LibRead](https://libread.com/)
- [lightnovelworld](https://www.lightnovelworld.com/)

### Cargo features

Each available backend matches a [cargo `feature`](https://doc.rust-lang.org/cargo/reference/features.html) that can be enabled or
disabled.

By default, only the *royalroad* and *freewebnovel* are enabled. *libread*
is disabled by default since (in my meager experience) it is simply a
different frontend for *freewebnovel*.

if you want all features, including the default ones:
```toml
# Cargo.toml
[dependencies]
libwebnovel = {version="*", features = ["all"]}
```

#### A note on Royal Road

RoyalRoad adds anti-theft text when getting chapters outside their
website. This is good to tackle malicious individuals seeking to profit of
someone else's work, but quite bad when downloading chapters for your
offline perusing, so this crates removes them. This is done by a helper
program, repeatedly requesting a chapter and comparing what text changes.
The list of changes is then saved to a file on the repository, which is
later included at build-time.

I have been running this helper binary to generate a list that did not seem
to grow any more, but RR may add more sentences in the future. If you spot
one of those, you can open an issue.

If you want to publish a merge request, that's even better, here's how to
run the helper script:

```txt
$ cargo run --features=helper_scripts --bin=rr-gen-anti-theft-list
```

You can then commit the resulting
`ressources/royalroad/known_anti-theft_sentences.txt` and send a merge
request.

### Crate features / Task list

- [ ] Find a way to handle something other than text content:
  - [ ] images
  - [ ] tables
  - [ ] chapter headers ?
  - [ ] chapter footers ?
- [ ] Add more backends:
  - [x] libread
  - [x] freewebnovel
  - [x] royalroad
  - [x] lightnovelworld
  - [ ] scribblehub - May be complicated because of cloudflare
  - [ ] suggestions?
- [ ] implement an `async` version to get a better throughput. May be
  important for images?
- [x] ~create a binary using this lib to save webnovels to disk. It may also
  serve as a sample implementation?~ See [libwebnovel-storage](https://crates.io/crates/libwebnovel-storage)
- [x] implement a way to get an [`Ordering`][std::cmp::Ordering] between
  chapters. That enables us to detect collisions and still sort chapters
  that may have their indexes altered, such as in the case of removal in the
  source.
- [x] Add a way to detect potential collisions without requesting each
  individual chapter.
- [x] Add a way to get the chapter url & parent fiction url from a given
  chapter.
- [x] ~maybe find a way to parse a chapter index/number as to not overwrite
  local files when chapters are deleted on the backend~ -> done via
  [`Backends::get_ordering_function`].
- [x] add a way to get the cover image of the fiction, for epub generation.

### Legal

Without explicit refutation in the header of any file in this repository,
all files in this repository are considered under the terms of the AGPL-3
license (of which a copy can be found in the LICENSE file at the root of
this repository) and bearing the mention "Copyright (c) 2024 paulollivier &
contributors".

Basically, please do not use this code without crediting its writer(s) or
for a commercial project.

License: AGPL-3.0-or-later
