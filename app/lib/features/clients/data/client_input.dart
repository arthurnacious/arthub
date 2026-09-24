import 'package:equatable/equatable.dart';

import 'client.dart';

const clientStatusOptions = ['active', 'prospect', 'inactive'];

/// The create/update payload - matches CreateClientDto/UpdateClientDto
/// (backend/src/Modules/Clients/Dto/) and the web frontend's
/// ClientFormValues (frontend/src/components/ClientForm.tsx). Distinct
/// from [Client] because a Client also carries server-assigned fields
/// (id, projects count) that a form never edits.
class ClientInput extends Equatable {
  const ClientInput({
    required this.company,
    required this.contact,
    required this.email,
    required this.status,
  });

  const ClientInput.blank()
      : company = '',
        contact = '',
        email = '',
        status = 'active';

  factory ClientInput.fromClient(Client client) => ClientInput(
        company: client.company,
        contact: client.contact ?? '',
        email: client.email ?? '',
        status: client.status,
      );

  final String company;
  final String contact;
  final String email;
  final String status;

  ClientInput copyWith({String? company, String? contact, String? email, String? status}) {
    return ClientInput(
      company: company ?? this.company,
      contact: contact ?? this.contact,
      email: email ?? this.email,
      status: status ?? this.status,
    );
  }

  Map<String, dynamic> toJson() => {
        'company': company,
        'contact': contact,
        'email': email,
        'status': status,
      };

  @override
  List<Object?> get props => [company, contact, email, status];
}
