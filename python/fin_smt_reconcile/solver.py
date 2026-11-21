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

    def reconcile_cross_footing(
        self,
        matrix: list[list[Decimal]],
        row_total_col: int = -1,
        col_total_row: int = -1,
    ) -> ReconciliationReport:
        rows = len(matrix)
        cols = len(matrix[0])
        r_tot = row_total_col if row_total_col >= 0 else cols + row_total_col
        c_tot = col_total_row if col_total_row >= 0 else rows + col_total_row

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

        # Soft observations: minimize count of perturbed cells
        for r in range(rows):
            for c in range(cols):
                obs = z3.RealVal(str(matrix[r][c]))
                opt.add_soft(vars_grid[r][c] == obs, weight=1)

        if opt.check() != z3.sat:
            raise ValueError("Accounting invariants are structurally unsatisfiable")

        model = opt.model()
        anomalies: list[CellAnomaly] = []
        repaired: list[list[Decimal]] = []

        for r in range(rows):
            out_row: list[Decimal] = []
            for c in range(cols):
                val_str = model[vars_grid[r][c]].as_decimal(6).rstrip("?")
                rep_dec = Decimal(val_str).quantize(Decimal("0.01"))
                obs_dec = matrix[r][c].quantize(Decimal("0.01"))
                if rep_dec != obs_dec:
                    anomalies.append(
                        CellAnomaly(row=r, col=c, observed=obs_dec, repaired=rep_dec)
                    )
                out_row.append(rep_dec)
            repaired.append(out_row)

        return ReconciliationReport(
            is_consistent=(len(anomalies) == 0),
            anomalies=anomalies,
            repaired_matrix=repaired,
        )
