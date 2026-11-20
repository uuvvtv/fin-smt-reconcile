from dataclasses import dataclass
from decimal import Decimal


@dataclass
class CellAnomaly:
    row: int
    col: int
    observed: Decimal
    repaired: Decimal


@dataclass
class ReconciliationReport:
    is_consistent: bool
    anomalies: list[CellAnomaly]
    repaired_matrix: list[list[Decimal]]
