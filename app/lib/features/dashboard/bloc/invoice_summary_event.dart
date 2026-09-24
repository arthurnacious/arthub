part of 'invoice_summary_bloc.dart';

sealed class InvoiceSummaryEvent extends Equatable {
  const InvoiceSummaryEvent();

  @override
  List<Object?> get props => [];
}

class InvoiceSummaryRequested extends InvoiceSummaryEvent {
  const InvoiceSummaryRequested();
}
