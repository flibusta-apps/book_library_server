from app.models import Sequence

from app.services.common import TRGMSearchService


class SequenceTGRMSearchService(TRGMSearchService):
    MODEL = Sequence
    FIELDS = [
        Sequence.Meta.table.c.name
    ]
    PREFETCH_RELATED = ["source"]
