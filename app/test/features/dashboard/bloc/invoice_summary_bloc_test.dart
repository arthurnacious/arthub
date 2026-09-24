import 'package:arthub/core/network/api_client.dart';
import 'package:arthub/core/network/api_exception.dart';
import 'package:arthub/core/storage/token_storage.dart';
import 'package:arthub/features/dashboard/bloc/invoice_summary_bloc.dart';
import 'package:arthub/features/invoices/data/invoice.dart';
import 'package:arthub/features/invoices/data/invoices_repository.dart';
import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';

final _dummyApiClient = ApiClient(tokenStorage: TokenStorage());

class _FakeInvoicesRepository extends InvoicesRepository {
  _FakeInvoicesRepository(this.invoices) : super(apiClient: _dummyApiClient);
  final List<Invoice> invoices;

  @override
  Future<List<Invoice>> fetchAllForSummary() async => invoices;
}

class _FailingInvoicesRepository extends InvoicesRepository {
  _FailingInvoicesRepository() : super(apiClient: _dummyApiClient);

  @override
  Future<List<Invoice>> fetchAllForSummary() async => throw const ApiException('boom', 500);
}

void main() {
  group('InvoiceSummaryBloc', () {
    blocTest<InvoiceSummaryBloc, InvoiceSummaryState>(
      'counts invoices per status and sums only the paid ones as revenue',
      build: () => InvoiceSummaryBloc(
        invoicesRepository: _FakeInvoicesRepository(const [
          Invoice(id: '1', status: 'paid', total: 500, paidAt: '2026-09-01'),
          Invoice(id: '2', status: 'paid', total: 300, paidAt: '2026-09-02'),
          Invoice(id: '3', status: 'sent', total: 999, paidAt: null),
        ]),
      ),
      act: (bloc) => bloc.add(const InvoiceSummaryRequested()),
      expect: () => [
        const InvoiceSummaryState(status: InvoiceSummaryStatus.loading),
        isA<InvoiceSummaryState>()
            .having((s) => s.status, 'status', InvoiceSummaryStatus.success)
            .having((s) => s.statusCounts['paid'], 'paid count', 2)
            .having((s) => s.statusCounts['sent'], 'sent count', 1)
            .having((s) => s.statusCounts['draft'], 'draft count (present, zero)', 0)
            .having((s) => s.totalPaidRevenue, 'totalPaidRevenue', 800),
      ],
    );

    blocTest<InvoiceSummaryBloc, InvoiceSummaryState>(
      'emits failure with the error message when the fetch throws',
      build: () => InvoiceSummaryBloc(invoicesRepository: _FailingInvoicesRepository()),
      act: (bloc) => bloc.add(const InvoiceSummaryRequested()),
      expect: () => [
        const InvoiceSummaryState(status: InvoiceSummaryStatus.loading),
        isA<InvoiceSummaryState>()
            .having((s) => s.status, 'status', InvoiceSummaryStatus.failure)
            .having((s) => s.errorMessage, 'errorMessage', 'boom'),
      ],
    );
  });
}
