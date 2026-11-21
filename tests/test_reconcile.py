from decimal import Decimal
from fin_smt_reconcile import MatrixReconciler, SpatialTableIndex, TextSpan


def test_cross_page_stitching_and_smt_repair():
    # Simulate a financial schedule split across page 0 and page 1
    spans = [
        # Page 0: Line item 1 + Line item 2
        TextSpan(0, 40.0, 720.0, 140.0, 732.0, "Software IP"),
        TextSpan(0, 220.0, 721.0, 270.0, 732.0, "120.50"),
        TextSpan(0, 320.0, 720.5, 370.0, 732.0, "30.00"),
        TextSpan(0, 420.0, 721.0, 470.0, 732.0, "150.50"),

        TextSpan(0, 42.0, 745.0, 145.0, 757.0, "Patents"),
        TextSpan(0, 221.0, 746.0, 271.0, 757.0, "88.00"),  # Corrupted OCR: true value is 80.00
        TextSpan(0, 319.0, 745.0, 369.0, 757.0, "20.00"),
        TextSpan(0, 421.0, 745.5, 471.0, 757.0, "100.00"),

        # Page 1: Total row after page break
        TextSpan(1, 41.0, 85.0, 110.0, 97.0, "Total"),
        TextSpan(1, 220.5, 85.5, 270.5, 97.0, "200.50"),
        TextSpan(1, 320.0, 85.0, 370.0, 97.0, "50.00"),
        TextSpan(1, 420.0, 85.5, 470.0, 97.0, "250.50"),
    ]

    index = SpatialTableIndex(spans, page_stride=842.0)
    grid = index.reconstruct_grid(y_tol=4.0, x_col_tol=15.0)

    assert len(grid) == 3
    assert grid[2][0] == "Total"

    numeric_matrix = [[Decimal(cell) for cell in row[1:]] for row in grid]
    reconciler = MatrixReconciler(tolerance="0.01")
    report = reconciler.reconcile_cross_footing(numeric_matrix)

    assert not report.is_consistent
    assert len(report.anomalies) == 1
    assert report.anomalies[0].row == 1
    assert report.anomalies[0].col == 0
    assert report.anomalies[0].observed == Decimal("88.00")
    assert report.anomalies[0].repaired == Decimal("80.00")
