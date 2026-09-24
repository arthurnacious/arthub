/// Mirrors the web frontend's ApiError (frontend/src/lib/api/http.ts):
/// the backend returns either `{"error": "message"}` or
/// `{"errors": {"field": ["message", ...]}}` on a non-2xx response - this
/// normalizes both into a single readable [message].
class ApiException implements Exception {
  const ApiException(this.message, this.statusCode);

  final String message;
  final int statusCode;

  factory ApiException.fromResponseBody(Object? body, int statusCode) {
    if (body is Map<String, dynamic>) {
      final error = body['error'];
      if (error is String) return ApiException(error, statusCode);

      final errors = body['errors'];
      if (errors is Map<String, dynamic>) {
        final messages = errors.values
            .whereType<List<dynamic>>()
            .expand((list) => list)
            .map((e) => e.toString());
        if (messages.isNotEmpty) {
          return ApiException(messages.join(', '), statusCode);
        }
      }
    }

    return ApiException('Request failed with status $statusCode', statusCode);
  }

  bool get isUnauthorized => statusCode == 401;
  bool get isNotFound => statusCode == 404;

  @override
  String toString() => message;
}
