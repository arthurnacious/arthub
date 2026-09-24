import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/network/api_exception.dart';
import '../../invoices/data/invoices_repository.dart';

part 'invoice_summary_event.dart';
part 'invoice_summary_state.dart';

/// Backs the dashboard's "invoice_status" and "revenue" panels - unlike
/// the web frontend, which fetches invoices independently per panel
/// (RevenuePanel.tsx and InvoiceStatusPanel.tsx each call fetchInvoices
/// on their own, deliberately, for isolation - see their comments), this
/// combines both into one fetch/bloc. Mobile data is worth being more
/// careful with than a desktop browser tab, and both summaries come from
/// the exact same invoice list anyway.
class InvoiceSummaryBloc extends Bloc<InvoiceSummaryEvent, InvoiceSummaryState> {
  InvoiceSummaryBloc({required InvoicesRepository invoicesRepository})
      : _invoicesRepository = invoicesRepository,
        super(const InvoiceSummaryState()) {
    on<InvoiceSummaryRequested>(_onRequested);
  }

  final InvoicesRepository _invoicesRepository;

  Future<void> _onRequested(
    InvoiceSummaryRequested event,
    Emitter<InvoiceSummaryState> emit,
  ) async {
    emit(state.copyWith(status: InvoiceSummaryStatus.loading));

    try {
      final invoices = await _invoicesRepository.fetchAllForSummary();

      final counts = <String, int>{for (final status in invoiceStatusOrder) status: 0};
      var totalPaid = 0.0;
      for (final invoice in invoices) {
        counts[invoice.status] = (counts[invoice.status] ?? 0) + 1;
        if (invoice.paidAt != null) totalPaid += invoice.total;
      }

      emit(state.copyWith(
        status: InvoiceSummaryStatus.success,
        statusCounts: counts,
        totalPaidRevenue: totalPaid,
      ));
    } on ApiException catch (error) {
      emit(state.copyWith(status: InvoiceSummaryStatus.failure, errorMessage: error.message));
    }
  }
}
