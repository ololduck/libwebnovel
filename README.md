# libwebnovel

![docs.rs](https://img.shields.io/docsrs/libwebnovel)

This crate deals with webnovels. You can see it as a way to access different
webnovel hosting sites and be able to get their contents.

Since there are times we don't have Internet access, such as when riding
some trains, downloading to disk in a convenient format seems the way to go.

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
want to implement an other backend (don't forget to share it with the [main repository](https://codeberg.org/paulollivier/libwebnovel)!).

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
  - [ ] suggestions?
- [ ] implement an `async` version to get a better throughput. May be
  important for images?
- [ ] create a binary using this lib to save webnovels to disk. It may also
  serve as a sample implementation?
- [x] implement a way to get an [`Ordering`][std::cmp::Ordering] between
  chapters. That enables us to detect collisions and still sort chapters
  that may have their indexes altered, such as in the case of removal in the
  source.
- [x] Add a way to get the chapter url & parent fiction url from a given
  chapter.
- [x] ~maybe find a way to parse a chapter index/number as to not overwrite
  local files when chapters are deleted on the backend~ -> done via
  [`Backends::get_ordering_function`].

### Legal

Without explicit refutation in the header of any file in this repository,
all files in this repository are considered under the terms of the AGPL-3
license (of which a copy can be found in the LICENSE file at the root of
this repository) and bearing the mention "Copyright (c) 2024 paulollivier &
contributors".

Basically, please do not use this code without crediting its writer(s) or
for a commercial project.

License: AGPL-3.0-or-later
