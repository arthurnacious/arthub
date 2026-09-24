import 'package:arthub/core/network/api_exception.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('ApiException.fromResponseBody', () {
    test('reads a single {"error": "..."} message', () {
      final exception = ApiException.fromResponseBody({'error': 'Invalid credentials'}, 401);

      expect(exception.message, 'Invalid credentials');
      expect(exception.statusCode, 401);
      expect(exception.isUnauthorized, isTrue);
    });

    test('joins a {"errors": {field: [messages]}} validation body', () {
      final exception = ApiException.fromResponseBody({
        'errors': {
          'email': ['must be a valid email'],
          'name': ['is required'],
        },
      }, 422);

      expect(exception.message, 'must be a valid email, is required');
    });

    test('falls back to a generic message when the body has neither shape', () {
      final exception = ApiException.fromResponseBody(null, 500);

      expect(exception.message, 'Request failed with status 500');
    });

    test('isNotFound is true only for a 404', () {
      expect(ApiException.fromResponseBody(null, 404).isNotFound, isTrue);
      expect(ApiException.fromResponseBody(null, 500).isNotFound, isFalse);
    });
  });
}
