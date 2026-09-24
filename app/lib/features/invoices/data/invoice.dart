import 'package:equatable/equatable.dart';

/// Matches InvoicesController::present() - see
/// backend/src/Modules/Invoices/InvoicesController.php - trimmed to just
/// what the dashboard's invoice summary needs (status + total + paidAt).
/// A full Invoice model (matching the web frontend's Invoice interface)
/// is future work, alongside a real Invoices feature.
class Invoice extends Equatable {
  const Invoice({required this.id, required this.status, required this.total, required this.paidAt});

  factory Invoice.fromJson(Map<String, dynamic> json) => Invoice(
        id: json['id'] as String,
        status: json['status'] as String,
        total: (json['total'] as num).toDouble(),
        paidAt: json['paidAt'] as String?,
      );

  final String id;
  final String status; // draft | sent | paid | overdue | cancelled
  final double total;
  final String? paidAt;

  @override
  List<Object?> get props => [id, status, total, paidAt];
}
