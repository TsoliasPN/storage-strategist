export type RiskLevel = "low" | "medium" | "high";

export interface ScanRequest {
  scan_id?: string;
  paths: string[];
  output?: string;
  max_depth?: number;
  excludes: string[];
  dedupe: boolean;
  dedupe_min_size: number;
  backend: "native" | "pdu_library";
  progress: boolean;
  min_ratio?: number;
  emit_progress_events: boolean;
  progress_interval_ms: number;
}

export interface ScanProgressEvent {
  seq: number;
  scan_id: string;
  phase:
    | "enumerating_disks"
    | "walking_files"
    | "categorizing"
    | "dedupe"
    | "recommending"
    | "done";
  current_path?: string;
  scanned_files: number;
  scanned_bytes: number;
  errors: number;
  timestamp: string;
}

export type ScanSessionStatus = "running" | "completed" | "cancelled" | "failed";

export interface ScanSessionSnapshot {
  scan_id: string;
  status: ScanSessionStatus;
  report_path?: string;
  error?: string;
  total_events: number;
}

export interface Recommendation {
  id: string;
  title: string;
  rationale: string;
  confidence: number;
  target_mount?: string;
  policy_safe: boolean;
  policy_rules_applied: string[];
  policy_rules_blocked: string[];
  risk_level: RiskLevel;
}

export interface PathStats {
  root_path: string;
  file_count: number;
  directory_count: number;
  total_size_bytes: number;
}

export interface CategorySuggestion {
  target: string;
  category: string;
  confidence: number;
  rationale: string;
}

export interface DuplicateGroup {
  size_bytes: number;
  hash: string;
  total_wasted_bytes: number;
  files: Array<{ path: string }>;
  intent?: { label: string; rationale: string };
}

export interface RuleTrace {
  rule_id: string;
  status: string;
  detail: string;
}

export interface DiskRoleHint {
  role: string;
  confidence: number;
  evidence: string[];
}

export interface DiskInfo {
  name: string;
  mount_point: string;
  locality_class: string;
  performance_class: string;
  is_os_drive: boolean;
  eligible_for_local_target: boolean;
  ineligible_reasons: string[];
  role_hint: DiskRoleHint;
}

export interface Report {
  scan_id: string;
  report_version: string;
  generated_at: string;
  disks: DiskInfo[];
  paths?: PathStats[];
  categories?: CategorySuggestion[];
  duplicates?: DuplicateGroup[];
  recommendations: Recommendation[];
  rule_traces?: RuleTrace[];
  warnings: string[];
}

export interface DoctorInfo {
  os: string;
  arch: string;
  current_dir?: string;
  os_mount?: string;
  read_only_mode: boolean;
  disks: DiskInfo[];
  notes: string[];
}

export interface RecommendationBundle {
  recommendations: Recommendation[];
}
