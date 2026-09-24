part of 'invoice_summary_bloc.dart';

enum InvoiceSummaryStatus { loading, success, failure }

// Every invoice status, in the order the dashboard displays them - same
// order/set as the web frontend's INVOICE_STATUS_TONE
// (frontend/src/lib/api/invoices.ts).
const invoiceStatusOrder = ['draft', 'sent', 'paid', 'overdue', 'cancelled'];

class InvoiceSummaryState extends Equatable {
  const InvoiceSummaryState({
    this.status = InvoiceSummaryStatus.loading,
    this.statusCounts = const {},
    this.totalPaidRevenue = 0,
    this.errorMessage,
  });

  final InvoiceSummaryStatus status;
  final Map<String, int> statusCounts;
  final double totalPaidRevenue;
  final String? errorMessage;

  InvoiceSummaryState copyWith({
    InvoiceSummaryStatus? status,
    Map<String, int>? statusCounts,
    double? totalPaidRevenue,
    String? errorMessage,
  }) {
    return InvoiceSummaryState(
      status: status ?? this.status,
      statusCounts: statusCounts ?? this.statusCounts,
      totalPaidRevenue: totalPaidRevenue ?? this.totalPaidRevenue,
      errorMessage: errorMessage,
    );
  }

  @override
  List<Object?> get props => [status, statusCounts, totalPaidRevenue, errorMessage];
}
