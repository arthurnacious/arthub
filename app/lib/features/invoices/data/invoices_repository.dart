import '../../../core/network/api_client.dart';
import 'invoice.dart';

/// Wraps /invoices - see backend/src/Modules/Invoices/InvoicesController.php.
/// Dashboard-only for now; a real Invoices feature is future work.
class InvoicesRepository {
  InvoicesRepository({required ApiClient apiClient}) : _apiClient = apiClient;

  final ApiClient _apiClient;

  // pageSize large enough to cover this app's scale in one request - same
  // "just fetch everything" convention the web frontend's dashboard
  // panels use (see frontend/src/components/dashboard/RevenuePanel.tsx),
  // not a paginated view.
  Future<List<Invoice>> fetchAllForSummary() async {
    final body = await _apiClient.get('/invoices', query: {'page': 1, 'pageSize': 200});
    return (body['data'] as List<dynamic>)
        .map((json) => Invoice.fromJson(json as Map<String, dynamic>))
        .toList();
  }
}
