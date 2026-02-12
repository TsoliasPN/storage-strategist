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
  incremental_cache?: boolean;
  cache_dir?: string;
  cache_ttl_seconds?: number;
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
  estimated_impact: EstimatedImpact;
  risk_level: RiskLevel;
}

export interface EstimatedImpact {
  space_saving_bytes?: number | null;
  performance?: string | null;
  risk_notes?: string | null;
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
  status: "emitted" | "skipped" | "rejected";
  detail: string;
  recommendation_id?: string | null;
  confidence?: number | null;
}

export interface PolicyDecision {
  policy_id: string;
  recommendation_id: string;
  action: "allowed" | "blocked";
  rationale: string;
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
  policy_decisions?: PolicyDecision[];
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

export interface ScenarioRiskMix {
  low: number;
  medium: number;
  high: number;
}

export interface ScenarioProjection {
  scenario_id: string;
  title: string;
  strategy: "conservative" | "balanced" | "aggressive";
  recommendation_ids: string[];
  recommendation_count: number;
  projected_space_saving_bytes: number;
  risk_mix: ScenarioRiskMix;
  blocked_recommendation_count: number;
  notes: string[];
}

export interface ScenarioPlan {
  generated_at: string;
  scan_id: string;
  assumptions: string[];
  scenarios: ScenarioProjection[];
}

export interface DiagnosticsEnvironment {
  os: string;
  arch: string;
  current_dir?: string;
  os_mount?: string;
  read_only_mode: boolean;
  app_version: string;
}

export interface DiagnosticsBundle {
  generated_at: string;
  source_report_path?: string;
  report: Report;
  doctor: DoctorInfo;
  environment: DiagnosticsEnvironment;
}
