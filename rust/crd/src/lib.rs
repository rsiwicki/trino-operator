pub mod authorization;
pub mod discovery;
pub mod error;

use crate::authorization::Authorization;

use serde::{Deserialize, Serialize};
use stackable_operator::kube::runtime::reflector::ObjectRef;
use stackable_operator::kube::CustomResource;
use stackable_operator::product_config_utils::{ConfigError, Configuration};
use stackable_operator::role_utils::Role;
use stackable_operator::role_utils::RoleGroupRef;
use stackable_operator::schemars::{self, JsonSchema};
use std::collections::BTreeMap;
use strum_macros::Display;
use strum_macros::EnumIter;

pub const APP_NAME: &str = "trino";
pub const APP_PORT: u16 = 8080;

// file names
pub const CONFIG_PROPERTIES: &str = "config.properties";
pub const JVM_CONFIG: &str = "jvm.config";
pub const NODE_PROPERTIES: &str = "node.properties";
pub const LOG_PROPERTIES: &str = "log.properties";
pub const PASSWORD_AUTHENTICATOR_PROPERTIES: &str = "password-authenticator.properties";
pub const PASSWORD_DB: &str = "password.db";
pub const CERTIFICATE_PEM: &str = "clustercoord.pem";
pub const HIVE_PROPERTIES: &str = "hive.properties";
// node.properties
pub const NODE_ENVIRONMENT: &str = "node.environment";
pub const NODE_ID: &str = "node.id";
pub const NODE_DATA_DIR: &str = "node.data-dir";
// config.properties
pub const COORDINATOR: &str = "coordinator";
pub const HTTP_SERVER_HTTP_PORT: &str = "http-server.http.port";
pub const HTTP_SERVER_HTTPS_PORT: &str = "http-server.https.port";
pub const HTTP_SERVER_HTTPS_ENABLED: &str = "http-server.https.enabled";
pub const HTTP_SERVER_KEYSTORE_PATH: &str = "http-server.https.keystore.path";
pub const HTTP_SERVER_AUTHENTICATION_TYPE: &str = "http-server.authentication.type";
pub const HTTP_SERVER_AUTHENTICATION_TYPE_PASSWORD: &str = "PASSWORD";
pub const QUERY_MAX_MEMORY: &str = "query.max-memory";
pub const QUERY_MAX_MEMORY_PER_NODE: &str = "query.max-memory-per-node";
pub const QUERY_MAX_TOTAL_MEMORY_PER_NODE: &str = "query.max-total-memory-per-node";
pub const DISCOVERY_URI: &str = "discovery.uri";
// password-authenticator.properties
pub const PASSWORD_AUTHENTICATOR_NAME: &str = "password-authenticator.name";
pub const PASSWORD_AUTHENTICATOR_NAME_FILE: &str = "file"; // the value of the property above
pub const FILE_PASSWORD_FILE: &str = "file.password-file";
// file content keys
pub const PW_FILE_CONTENT_MAP_KEY: &str = "pwFileContent";
pub const CERT_FILE_CONTENT_MAP_KEY: &str = "serverCertificate";
// hive.properties
pub const S3_ENDPOINT: &str = "hive.s3.endpoint";
pub const S3_ACCESS_KEY: &str = "hive.s3.aws-access-key";
pub const S3_SECRET_KEY: &str = "hive.s3.aws-secret-key";
pub const S3_SSL_ENABLED: &str = "hive.s3.ssl.enabled";
pub const S3_PATH_STYLE_ACCESS: &str = "hive.s3.path-style-access";
// log.properties
pub const IO_TRINO: &str = "io.trino";
// jvm.config
pub const METRICS_PORT_PROPERTY: &str = "metricsPort";
// port names
pub const METRICS_PORT: &str = "metrics";
pub const HTTP_PORT: &str = "http";
pub const HTTPS_PORT: &str = "https";
// config dir
pub const CONFIG_DIR_NAME: &str = "/stackable/conf";

#[derive(Clone, CustomResource, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[kube(
    group = "trino.stackable.tech",
    version = "v1alpha1",
    kind = "TrinoCluster",
    plural = "trinoclusters",
    shortname = "trino",
    namespaced,
    crates(
        kube_core = "stackable_operator::kube::core",
        k8s_openapi = "stackable_operator::k8s_openapi",
        schemars = "stackable_operator::schemars"
    )
)]
#[kube(status = "TrinoClusterStatus")]
#[serde(rename_all = "camelCase")]
pub struct TrinoClusterSpec {
    /// Emergency stop button, if `true` then all pods are stopped without affecting configuration (as setting `replicas` to `0` would)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stopped: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub node_environment: String,
    pub hive_reference: ClusterRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opa: Option<ClusterRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authorization: Option<Authorization>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_connection: Option<S3Connection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coordinators: Option<Role<TrinoConfig>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workers: Option<Role<TrinoConfig>>,
}

#[derive(
    Clone, Debug, Deserialize, Display, EnumIter, Eq, Hash, JsonSchema, PartialEq, Serialize,
)]
pub enum TrinoRole {
    #[strum(serialize = "coordinator")]
    Coordinator,
    #[strum(serialize = "worker")]
    Worker,
}

impl TrinoRole {
    /// Returns the container start command for a Trino node.
    pub fn get_command(&self) -> Vec<String> {
        vec![
            "bin/launcher".to_string(),
            "run".to_string(),
            format!("--etc-dir={}", CONFIG_DIR_NAME),
        ]
    }
}

// TODO: move to operator-rs? Used for hive, opa, zookeeper ...
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClusterRef {
    pub name: String,
    pub namespace: String,
    pub chroot: Option<String>,
}

// TODO: move to operator-rs? Copied from hive operator.
/// Contains all the required connection information for S3.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct S3Connection {
    pub end_point: String,
    pub access_key: String,
    pub secret_key: String,
    pub ssl_enabled: bool,
    pub path_style_access: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrinoConfig {
    // config.properties
    pub coordinator: Option<bool>,
    pub http_server_http_port: Option<u16>,
    pub http_server_https_port: Option<u16>,
    pub query_max_memory: Option<String>,
    pub query_max_memory_per_node: Option<String>,
    pub query_max_total_memory_per_node: Option<String>,
    // node.properties
    pub node_data_dir: Option<String>,
    // log.properties
    pub io_trino: Option<String>,
    // jvm.config
    pub metrics_port: Option<u16>,
    // TLS certificate
    pub server_certificate: Option<String>,
    // password file auth
    pub password_file_content: Option<String>,
}

impl Configuration for TrinoConfig {
    type Configurable = TrinoCluster;

    fn compute_env(
        &self,
        _resource: &Self::Configurable,
        _role_name: &str,
    ) -> Result<BTreeMap<String, Option<String>>, ConfigError> {
        Ok(BTreeMap::new())
    }

    fn compute_cli(
        &self,
        _resource: &Self::Configurable,
        _role_name: &str,
    ) -> Result<BTreeMap<String, Option<String>>, ConfigError> {
        Ok(BTreeMap::new())
    }

    fn compute_files(
        &self,
        resource: &Self::Configurable,
        _role_name: &str,
        file: &str,
    ) -> Result<BTreeMap<String, Option<String>>, ConfigError> {
        let mut result = BTreeMap::new();

        match file {
            NODE_PROPERTIES => {
                result.insert(
                    NODE_ENVIRONMENT.to_string(),
                    Some(resource.spec.node_environment.clone()),
                );

                if let Some(node_data_dir) = &self.node_data_dir {
                    result.insert(NODE_DATA_DIR.to_string(), Some(node_data_dir.to_string()));
                }
            }
            CONFIG_PROPERTIES => {
                if let Some(coordinator) = &self.coordinator {
                    result.insert(COORDINATOR.to_string(), Some(coordinator.to_string()));
                }
                if let Some(http_server_http_port) = &self.http_server_http_port {
                    result.insert(
                        HTTP_SERVER_HTTP_PORT.to_string(),
                        Some(http_server_http_port.to_string()),
                    );
                }

                if let Some(query_max_memory) = &self.query_max_memory {
                    result.insert(
                        QUERY_MAX_MEMORY.to_string(),
                        Some(query_max_memory.to_string()),
                    );
                }

                if let Some(query_max_memory_per_node) = &self.query_max_memory_per_node {
                    result.insert(
                        QUERY_MAX_MEMORY_PER_NODE.to_string(),
                        Some(query_max_memory_per_node.to_string()),
                    );
                }

                if let Some(query_max_total_memory_per_node) = &self.query_max_total_memory_per_node
                {
                    result.insert(
                        QUERY_MAX_TOTAL_MEMORY_PER_NODE.to_string(),
                        Some(query_max_total_memory_per_node.to_string()),
                    );
                }

                // if a certificate is provided, we enable TLS
                if self.server_certificate.is_some() {
                    result.insert(
                        HTTP_SERVER_HTTPS_ENABLED.to_string(),
                        Some(true.to_string()),
                    );
                    result.insert(
                        HTTP_SERVER_KEYSTORE_PATH.to_string(),
                        Some(format!("{}/{}", CONFIG_DIR_NAME, CERTIFICATE_PEM)),
                    );
                    if let Some(https_port) = &self.http_server_https_port {
                        result.insert(
                            HTTP_SERVER_HTTPS_PORT.to_string(),
                            Some(https_port.to_string()),
                        );
                    }
                }

                if self.password_file_content.is_some() {
                    result.insert(
                        HTTP_SERVER_AUTHENTICATION_TYPE.to_string(),
                        Some(HTTP_SERVER_AUTHENTICATION_TYPE_PASSWORD.to_string()),
                    );
                }
            }
            PASSWORD_AUTHENTICATOR_PROPERTIES => {
                if self.password_file_content.is_some() {
                    result.insert(
                        PASSWORD_AUTHENTICATOR_NAME.to_string(),
                        Some(PASSWORD_AUTHENTICATOR_NAME_FILE.to_string()),
                    );
                    result.insert(
                        FILE_PASSWORD_FILE.to_string(),
                        Some(format!("{}/{}", CONFIG_DIR_NAME, PASSWORD_DB)),
                    );
                }
            }
            PASSWORD_DB => {
                if let Some(pw_file_content) = &self.password_file_content {
                    result.insert(
                        PW_FILE_CONTENT_MAP_KEY.to_string(),
                        Some(pw_file_content.to_string()),
                    );
                }
            }
            CERTIFICATE_PEM => {
                if let Some(cert) = &self.server_certificate {
                    result.insert(
                        CERT_FILE_CONTENT_MAP_KEY.to_string(),
                        Some(cert.to_string()),
                    );
                }
            }
            HIVE_PROPERTIES => {
                if let Some(s3_connection) = &resource.spec.s3_connection {
                    result.insert(
                        S3_ENDPOINT.to_string(),
                        Some(s3_connection.end_point.clone()),
                    );

                    result.insert(
                        S3_ACCESS_KEY.to_string(),
                        Some(s3_connection.access_key.clone()),
                    );

                    result.insert(
                        S3_SECRET_KEY.to_string(),
                        Some(s3_connection.secret_key.clone()),
                    );

                    result.insert(
                        S3_SSL_ENABLED.to_string(),
                        Some(s3_connection.ssl_enabled.to_string()),
                    );

                    result.insert(
                        S3_PATH_STYLE_ACCESS.to_string(),
                        Some(s3_connection.path_style_access.to_string()),
                    );
                }
            }
            JVM_CONFIG => {
                if let Some(metrics_port) = self.metrics_port {
                    result.insert(
                        METRICS_PORT_PROPERTY.to_string(),
                        Some(metrics_port.to_string()),
                    );
                }
            }
            LOG_PROPERTIES => {
                if let Some(io_trino) = &self.io_trino {
                    result.insert(IO_TRINO.to_string(), Some(io_trino.to_string()));
                }
            }
            _ => {}
        }

        Ok(result)
    }
}

impl TrinoCluster {
    /// The name of the role-level load-balanced Kubernetes `Service` for the worker nodes
    pub fn worker_role_service_name(&self) -> Option<String> {
        self.metadata
            .name
            .as_ref()
            .map(|name| format!("{}-worker", name))
    }

    /// The name of the role-level load-balanced Kubernetes `Service` for the coordinator nodes
    pub fn coordinator_role_service_name(&self) -> Option<String> {
        self.metadata
            .name
            .as_ref()
            .map(|name| format!("{}-coordinator", name))
    }

    /// The fully-qualified domain name of the role-level load-balanced Kubernetes `Service`
    pub fn coordinator_role_service_fqdn(&self) -> Option<String> {
        Some(format!(
            "{}.{}.svc.cluster.local",
            self.coordinator_role_service_name()?,
            self.metadata.namespace.as_ref()?
        ))
    }

    /// The fully-qualified domain name of the role-level load-balanced Kubernetes `Service`
    pub fn worker_role_service_fqdn(&self) -> Option<String> {
        Some(format!(
            "{}.{}.svc.cluster.local",
            self.worker_role_service_name()?,
            self.metadata.namespace.as_ref()?
        ))
    }
    /// Metadata about a coordinator rolegroup
    pub fn coordinator_rolegroup_ref(
        &self,
        group_name: impl Into<String>,
    ) -> RoleGroupRef<TrinoCluster> {
        RoleGroupRef {
            cluster: ObjectRef::from_obj(self),
            role: TrinoRole::Coordinator.to_string(),
            role_group: group_name.into(),
        }
    }

    /// Metadata about a worker rolegroup
    pub fn worker_rolegroup_ref(
        &self,
        group_name: impl Into<String>,
    ) -> RoleGroupRef<TrinoCluster> {
        RoleGroupRef {
            cluster: ObjectRef::from_obj(self),
            role: TrinoRole::Worker.to_string(),
            role_group: group_name.into(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrinoClusterStatus {}
