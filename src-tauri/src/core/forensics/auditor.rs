// src-tauri/src/core/forensics/auditor.rs

use serde::Serialize;

#[derive(Serialize)]
pub struct RiskReport {
    pub token_id: String,
    pub application_name: String,
    pub risk_level: String, // "CRITICAL", "HIGH", "MEDIUM", "LOW"
    pub risk_score: u32,
    pub findings: Vec<String>,
}

pub struct IntegrationAuditor;

impl IntegrationAuditor {
    /// Analyzes the risk of an authorized OAuth2 application based on its granted scopes.
    pub fn audit_app(token_json: &serde_json::Value) -> RiskReport {
        let app_name = token_json["application"]["name"]
            .as_str()
            .unwrap_or("Unknown App")
            .to_string();
        let token_id = token_json["id"].as_str().unwrap_or_default().to_string();
        let scopes = token_json["scopes"].as_array();

        let mut score = 0;
        let mut findings = Vec::new();

        if let Some(scope_arr) = scopes {
            for s in scope_arr {
                if let Some(scope_str) = s.as_str() {
                    match scope_str {
                        "messages.read" => {
                            score += 40;
                            findings.push("Can read your private messages.".to_string());
                        }
                        "guilds.join" => {
                            score += 30;
                            findings.push("Can force you to join servers.".to_string());
                        }
                        "rpc" => {
                            score += 50;
                            findings.push("Full RPC control over your desktop client.".to_string());
                        }
                        "email" => {
                            score += 10;
                            findings.push("Can see your registered email address.".to_string());
                        }
                        "connections" => {
                            score += 20;
                            findings.push("Can see your linked 3rd-party accounts.".to_string());
                        }
                        _ => {}
                    }
                }
            }
        }

        let risk_level = if score >= 70 {
            "CRITICAL"
        } else if score >= 40 {
            "HIGH"
        } else if score >= 20 {
            "MEDIUM"
        } else {
            "LOW"
        };

        RiskReport {
            token_id,
            application_name: app_name,
            risk_level: risk_level.to_string(),
            risk_score: score,
            findings,
        }
    }
}
