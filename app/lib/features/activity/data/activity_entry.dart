import 'package:equatable/equatable.dart';

/// Matches ActivityController::index()'s raw row shape - see
/// backend/src/Modules/Activity/ActivityController.php. Unlike most
/// other endpoints, this one returns the DB columns directly
/// (snake_case) rather than going through a present() mapper - see
/// frontend/src/lib/api/activity.ts's ActivityEntry for the same shape
/// on the web side.
class ActivityEntry extends Equatable {
  const ActivityEntry({
    required this.id,
    required this.action,
    required this.description,
    required this.subjectType,
    required this.subjectId,
    required this.createdAt,
  });

  factory ActivityEntry.fromJson(Map<String, dynamic> json) => ActivityEntry(
        id: json['id'] as String,
        action: json['action'] as String,
        description: json['description'] as String,
        subjectType: json['subject_type'] as String?,
        subjectId: json['subject_id'] as String?,
        createdAt: json['created_at'] as String,
      );

  final String id;
  final String action;
  final String description;
  final String? subjectType;
  final String? subjectId;
  final String createdAt;

  /// Where tapping this entry should navigate, if anywhere - mirrors the
  /// web frontend's activityLink() (frontend/src/lib/activity-link.ts).
  /// Only 'client' has a screen to link to so far; every other
  /// subject_type (or none, e.g. auth events) is just non-interactive
  /// text.
  bool get linksToClient => subjectType == 'client' && subjectId != null;

  @override
  List<Object?> get props => [id, action, description, subjectType, subjectId, createdAt];
}
