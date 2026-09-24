part of 'client_create_bloc.dart';

enum ClientCreateStatus { editing, submitting, failure, success }

class ClientCreateState extends Equatable {
  const ClientCreateState({
    this.status = ClientCreateStatus.editing,
    this.form = const ClientInput.blank(),
    this.errorMessage,
  });

  final ClientCreateStatus status;
  final ClientInput form;
  final String? errorMessage;

  ClientCreateState copyWith({
    ClientCreateStatus? status,
    ClientInput? form,
    String? errorMessage,
  }) {
    return ClientCreateState(
      status: status ?? this.status,
      form: form ?? this.form,
      errorMessage: errorMessage,
    );
  }

  @override
  List<Object?> get props => [status, form, errorMessage];
}
