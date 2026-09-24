import 'package:equatable/equatable.dart';

/// Matches AuthController::present() - see
/// backend/src/Modules/Auth/AuthController.php - and the web frontend's
/// Profile interface (frontend/src/lib/api/auth.ts).
class User extends Equatable {
  const User({
    required this.id,
    required this.name,
    required this.email,
    required this.role,
  });

  factory User.fromJson(Map<String, dynamic> json) => User(
        id: json['id'] as String,
        name: json['name'] as String,
        email: json['email'] as String,
        role: json['role'] as String,
      );

  final String id;
  final String name;
  final String email;
  final String role;

  @override
  List<Object?> get props => [id, name, email, role];
}
