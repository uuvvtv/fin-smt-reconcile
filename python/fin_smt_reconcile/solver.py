from dataclasses import dataclass
from decimal import Decimal
import z3


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


class MatrixReconciler:
    """
    Encodes 2D financial schedules (with row subtotals and column totals) into
    a Z3 SMT optimization model to isolate and repair OCR / LLM extraction drift.
    """

    def __init__(self, tolerance: str = "0.01") -> None:
        self.tolerance = z3.RealVal(tolerance)

    def _build_optimizer_with_hard_constraints(
        self,
        matrix: list[list[Decimal]],
        r_tot: int,
        c_tot: int,
    ) -> tuple[z3.Optimize, list[list[z3.ArithRef]]]:
        rows = len(matrix)
        cols = len(matrix[0])

        opt = z3.Optimize()
        vars_grid: list[list[z3.ArithRef]] = [
            [z3.Real(f"cell_{r}_{c}") for c in range(cols)] for r in range(rows)
        ]

        # Hard accounting invariants: horizontal row sums
        for r in range(rows):
            lhs = z3.Sum([vars_grid[r][c] for c in range(cols) if c != r_tot])
            rhs = vars_grid[r][r_tot]
            opt.add(lhs - rhs <= self.tolerance)
            opt.add(rhs - lhs <= self.tolerance)

        # Hard accounting invariants: vertical column sums
        for c in range(cols):
            lhs = z3.Sum([vars_grid[r][c] for r in range(rows) if r != c_tot])
            rhs = vars_grid[c_tot][c]
            opt.add(lhs - rhs <= self.tolerance)
            opt.add(rhs - lhs <= self.tolerance)

        return opt, vars_grid
