use axum::{extract::TypedHeader, headers::ContentType, routing, Router};
use include_dir::{include_dir, Dir, DirEntry};

const FRONTEND_ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../frontend/dist/assets");

fn walk_dir(dir: &'static Dir, mut router: Router) -> Router {
    for entry in dir.entries() {
        match entry {
            DirEntry::File(file) => {
                let path = file.path();
                let content = file.contents();
                let mime = mime_guess::from_path(path)
                    .first()
                    .unwrap_or(mime::APPLICATION_OCTET_STREAM);

                router = router.route(
                    &format!("/{}", path.display()),
                    routing::get(
                        move || async move { (TypedHeader(ContentType::from(mime)), content) },
                    ),
                );
            }
            DirEntry::Dir(dir) => {
                router = walk_dir(dir, router);
            }
        }
    }
    router
}

pub fn create_router() -> Router {
    walk_dir(&FRONTEND_ASSETS, Router::new())
}
