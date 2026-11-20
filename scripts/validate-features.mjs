#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const ALLOWED_STATUSES = [
  "DISCOVERED",
  "SPECIFIED",
  "IMPLEMENTING",
  "IMPLEMENTED",
  "TESTING",
  "APPROVED",
  "FAILED",
  "BLOCKED",
];

const ALLOWED_STATUS_SET = new Set(ALLOWED_STATUSES);

const REQUIRED_NON_EMPTY_FIELDS = [
  "id",
  "name",
  "category",
  "description",
  "expected_behavior",
  "implementation_path",
];

function isNonEmptyString(value) {
  return typeof value === "string" && value.trim().length > 0;
}

function isPlainObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

/**
 * Validates a parsed features.json object against schema and filesystem rules.
 * @param {unknown} data
 * @param {string} repoRoot
 * @returns {{
 *   valid: boolean,
 *   errors: string[],
 *   warnings: string[],
 *   statusCounts: Record<string, number>,
 *   unknownStatusCount: number,
 *   totalFeatures: number,
 * }}
 */
export function validateFeaturesDocument(data, repoRoot) {
  const errors = [];
  const warnings = [];
  const statusCounts = Object.fromEntries(ALLOWED_STATUSES.map((s) => [s, 0]));
  let unknownStatusCount = 0;

  if (!isPlainObject(data)) {
    errors.push("Root JSON value must be an object.");
    return {
      valid: false,
      errors,
      warnings,
      statusCounts,
      unknownStatusCount,
      totalFeatures: 0,
    };
  }

  if (!Array.isArray(data.features) || data.features.length === 0) {
    errors.push('Root object must contain a non-empty "features" array.');
    return {
      valid: false,
      errors,
      warnings,
      statusCounts,
      unknownStatusCount,
      totalFeatures: 0,
    };
  }

  const seenIds = new Map();

  data.features.forEach((feature, index) => {
    const label =
      isPlainObject(feature) && isNonEmptyString(feature.id)
        ? feature.id.trim()
        : `features[${index}]`;

    if (!isPlainObject(feature)) {
      errors.push(`${label}: feature entry must be a JSON object.`);
      return;
    }

    for (const field of REQUIRED_NON_EMPTY_FIELDS) {
      if (!isNonEmptyString(feature[field])) {
        errors.push(`${label}: "${field}" must be a non-empty string.`);
      }
    }

    if (isNonEmptyString(feature.id)) {
      const normalizedId = feature.id.trim();
      if (seenIds.has(normalizedId)) {
        errors.push(
          `${label}: duplicate feature id "${normalizedId}" (first seen at index ${seenIds.get(normalizedId)}).`,
        );
      } else {
        seenIds.set(normalizedId, index);
      }
    }

    if (!isNonEmptyString(feature.status) || !ALLOWED_STATUS_SET.has(feature.status)) {
      unknownStatusCount += 1;
      errors.push(
        `${label}: invalid status "${String(feature.status)}". Allowed: ${ALLOWED_STATUSES.join(", ")}.`,
      );
    } else {
      statusCounts[feature.status] += 1;
    }

    if (feature.status === "APPROVED") {
      const hasLastValidated =
        feature.last_validated !== null &&
        feature.last_validated !== undefined &&
        (typeof feature.last_validated !== "string" || feature.last_validated.trim().length > 0);

      if (!hasLastValidated) {
        errors.push(
          `${label}: feature with status "APPROVED" must have a non-null "last_validated" timestamp.`,
        );
      }

      if (!isNonEmptyString(feature.test_result)) {
        errors.push(
          `${label}: feature with status "APPROVED" must have a non-empty "test_result".`,
        );
      } else if (feature.test_result.trim().toLowerCase().startsWith("pending")) {
        errors.push(
          `${label}: feature with status "APPROVED" cannot have a Pending "test_result" ("${feature.test_result}").`,
        );
      }
    }

    if (isNonEmptyString(feature.implementation_path)) {
      const implPath = feature.implementation_path.trim();
      const resolvedImpl = path.resolve(repoRoot, implPath);
      if (!fs.existsSync(resolvedImpl)) {
        warnings.push(
          `${label}: implementation_path not found on disk: "${implPath}"`,
        );
      }
    }
  });

  return {
    valid: errors.length === 0,
    errors,
    warnings,
    statusCounts,
    unknownStatusCount,
    totalFeatures: data.features.length,
  };
}

function formatSummaryTable(result) {
  const statusColWidth = Math.max(
    "STATUS".length,
    ...ALLOWED_STATUSES.map((s) => s.length),
    "UNKNOWN".length,
    "TOTAL".length,
  );
  const countColWidth = 7;

  const border = `+${"-".repeat(statusColWidth + 2)}+${"-".repeat(countColWidth + 2)}+`;
  const row = (status, count) =>
    `| ${String(status).padEnd(statusColWidth)} | ${String(count).padStart(countColWidth)} |`;

  const lines = [
    border,
    row("STATUS", "COUNT"),
    border,
    ...ALLOWED_STATUSES.map((s) => row(s, result.statusCounts[s])),
  ];

  if (result.unknownStatusCount > 0) {
    lines.push(row("UNKNOWN", result.unknownStatusCount));
  }

  lines.push(border);
  lines.push(row("TOTAL", result.totalFeatures));
  lines.push(border);
  return lines.join("\n");
}

function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const repoRoot = path.resolve(scriptDir, "..");

  const args = process.argv.slice(2);
  const strict = args.includes("--strict");
  const positionalArg = args.find((a) => !a.startsWith("-"));
  const featuresFile = positionalArg
    ? path.resolve(process.cwd(), positionalArg)
    : path.join(repoRoot, "docs", "features.json");

  if (!fs.existsSync(featuresFile)) {
    console.error(`[ERROR] Features file not found: ${featuresFile}`);
    process.exit(1);
  }

  let parsed;
  try {
    const raw = fs.readFileSync(featuresFile, "utf8");
    parsed = JSON.parse(raw);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    console.error(`[ERROR] Failed to parse ${featuresFile} as JSON: ${message}`);
    process.exit(1);
  }

  const result = validateFeaturesDocument(parsed, repoRoot);

  console.log(`Feature Registry Validation: ${path.relative(repoRoot, featuresFile) || featuresFile}`);
  console.log("");
  console.log(formatSummaryTable(result));
  console.log("");

  if (result.warnings.length > 0) {
    console.warn(`Warnings (${result.warnings.length}):`);
    for (const warning of result.warnings) {
      console.warn(`  [WARN] ${warning}`);
    }
    console.log("");
  }

  if (result.errors.length > 0) {
    console.error(`Schema Errors (${result.errors.length}):`);
    for (const error of result.errors) {
      console.error(`  [ERROR] ${error}`);
    }
    console.log("");
    process.exit(1);
  }

  if (strict && result.warnings.length > 0) {
    console.error("Strict mode enabled: failing due to warnings.");
    process.exit(1);
  }

  console.log(
    `Validation PASSED (${result.totalFeatures} features checked, ${result.warnings.length} warning(s), 0 schema error(s)).`,
  );
  process.exit(0);
}

const isDirectExecution =
  typeof process.argv[1] === "string" &&
  path.resolve(process.argv[1]) === fileURLToPath(import.meta.url);

if (isDirectExecution) {
  main();
}
