from app.models import Author

from app.services.common import TRGMSearchService


class AuthorTGRMSearchService(TRGMSearchService):
    MODEL_CLASS = Author
    FIELDS = [
        Author.Meta.table.c.last_name,
        Author.Meta.table.c.first_name,
        Author.Meta.table.c.middle_name
    ]
    PREFETCH_RELATED = ["source"]
