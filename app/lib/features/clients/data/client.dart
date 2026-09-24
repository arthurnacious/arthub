import 'package:equatable/equatable.dart';

/// Matches ClientsController::present() - see
/// backend/src/Modules/Clients/ClientsController.php - and the web
/// frontend's Client interface (frontend/src/lib/mock-clients.ts).
class Client extends Equatable {
  const Client({
    required this.id,
    required this.company,
    required this.contact,
    required this.email,
    required this.status,
    required this.projects,
  });

  factory Client.fromJson(Map<String, dynamic> json) => Client(
        id: json['id'] as String,
        company: json['company'] as String,
        contact: json['contact'] as String?,
        email: json['email'] as String?,
        status: json['status'] as String,
        projects: json['projects'] as int,
      );

  final String id;
  final String company;
  final String? contact;
  final String? email;
  final String status; // active | prospect | inactive
  final int projects;

  @override
  List<Object?> get props => [id, company, contact, email, status, projects];
}
