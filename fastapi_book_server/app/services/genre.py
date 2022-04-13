from app.models import Genre
from app.services.common import MeiliSearchService


class GenreMeiliSearchService(MeiliSearchService):
    MODEL_CLASS = Genre
    PREFETCH_RELATED = ["source"]

    MS_INDEX_NAME = "genres"
    MS_INDEX_LANG_KEY = "langs"
