use byteorder::BigEndian;
use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::{
  messages::submessages::elements::{
    crypto_content::CryptoContent, crypto_footer::CryptoFooter, crypto_header::CryptoHeader,
  },
  security::{BinaryProperty, DataHolder, SecurityError},
  serialization::cdr_serializer::to_bytes,
  CdrDeserializer,
};
use super::types::{
  CryptoHandle, CryptoToken, CryptoTransformIdentifier, CryptoTransformKeyId, CryptoTransformKind,
};

const CRYPTO_TOKEN_CLASS_ID: &str = "DDS:Crypto:AES_GCM_GMAC";
const CRYPTO_TOKEN_KEYMAT_NAME: &str = "dds.cryp.keymat";

/// DDS:Crypto:AES-GCM-GMAC CryptoToken type from section 9.5.2.1 of the
/// Security specification (v. 1.1)
pub struct BuiltinCryptoToken {
  pub key_material: KeyMaterial_AES_GCM_GMAC,
}
impl TryFrom<CryptoToken> for BuiltinCryptoToken {
  type Error = SecurityError;
  fn try_from(value: CryptoToken) -> Result<Self, Self::Error> {
    let dh = value.data_holder;
    match (
      dh.class_id.as_str(),
      dh.properties.as_slice(),
      dh.binary_properties.as_slice(),
    ) {
      (CRYPTO_TOKEN_CLASS_ID, [], [bp0]) => {
        if bp0.name.eq(CRYPTO_TOKEN_KEYMAT_NAME) {
          Ok(Self {
            key_material: KeyMaterial_AES_GCM_GMAC::try_from(bp0.value.clone())?,
          })
        } else {
          Err(Self::Error {
            msg: format!(
              "The binary property of CryptoToken has the wrong name. Expected {}, got {}.",
              CRYPTO_TOKEN_KEYMAT_NAME, bp0.name
            ),
          })
        }
      }

      (CRYPTO_TOKEN_CLASS_ID, [], bps) => Err(Self::Error {
        msg: String::from(
          "CryptoToken has wrong binary_properties. Expected exactly 1 binary property.",
        ),
      }),
      (CRYPTO_TOKEN_CLASS_ID, ps, _) => Err(Self::Error {
        msg: String::from("CryptoToken has wrong properties. Expected properties to be empty."),
      }),

      (cid, _, _) => Err(Self::Error {
        msg: format!(
          "CryptoToken has wrong class_id. Expected {}, got {}",
          CRYPTO_TOKEN_CLASS_ID, cid
        ),
      }),
    }
  }
}

impl TryFrom<BuiltinCryptoToken> for CryptoToken {
  type Error = SecurityError;
  fn try_from(value: BuiltinCryptoToken) -> Result<Self, Self::Error> {
    Ok(CryptoToken {
      data_holder: DataHolder {
        class_id: String::from(CRYPTO_TOKEN_CLASS_ID),
        properties: Vec::new(),
        binary_properties: Vec::from([BinaryProperty {
          name: String::from(CRYPTO_TOKEN_KEYMAT_NAME),
          value: value.key_material.try_into()?,
          propagate: true,
        }]),
      },
    })
  }
}

impl From<KeyMaterial_AES_GCM_GMAC> for BuiltinCryptoToken {
  fn from(key_material: KeyMaterial_AES_GCM_GMAC) -> Self {
    Self { key_material }
  }
}
impl From<BuiltinCryptoToken> for KeyMaterial_AES_GCM_GMAC {
  fn from(BuiltinCryptoToken { key_material }: BuiltinCryptoToken) -> Self {
    key_material
  }
}

/// KeyMaterial_AES_GCM_GMAC type from section 9.5.2.1.1 of the Security
/// specification (v. 1.1)
#[allow(non_camel_case_types)] // We use the name from the spec
#[derive(Clone)]
pub struct KeyMaterial_AES_GCM_GMAC {
  pub transformation_kind: BuiltinCryptoTransformationKind,
  pub master_salt: Vec<u8>,
  pub sender_key_id: CryptoTransformKeyId,
  pub master_sender_key: Vec<u8>,
  pub receiver_specific_key_id: CryptoTransformKeyId,
  pub master_receiver_specific_key: Vec<u8>,
}

// Conversions from and into Bytes
impl TryFrom<Bytes> for KeyMaterial_AES_GCM_GMAC {
  type Error = SecurityError;
  fn try_from(value: Bytes) -> Result<Self, Self::Error> {
    // Deserialize CDR-formatted key material
    Serializable_KeyMaterial_AES_GCM_GMAC::deserialize(&mut CdrDeserializer::<
      BigEndian, /* TODO: What's the point of this constructor if we need to specify the byte
                  * order anyway */
    >::new_big_endian(value.as_ref()))
    .map_err(
      // Map deserialization error to SecurityError
      |e| Self::Error {
        msg: format!("Error deserializing KeyMaterial_AES_GCM_GMAC: {}", e),
      },
    )
    .and_then(KeyMaterial_AES_GCM_GMAC::try_from)
  }
}
impl TryFrom<KeyMaterial_AES_GCM_GMAC> for Bytes {
  type Error = SecurityError;
  fn try_from(keymat: KeyMaterial_AES_GCM_GMAC) -> Result<Self, Self::Error> {
    // Convert the key material to the serializable structure
    let serializable_keymat = Serializable_KeyMaterial_AES_GCM_GMAC::from(keymat);
    // Serialize
    to_bytes::<Serializable_KeyMaterial_AES_GCM_GMAC, BigEndian>(&serializable_keymat)
      .map(Bytes::from)
      .map_err(|e| Self::Error {
        msg: format!("Error serializing KeyMaterial_AES_GCM_GMAC: {}", e),
      })
  }
}

//Conversions from and into CryptoHandle
impl TryFrom<CryptoHandle> for KeyMaterial_AES_GCM_GMAC {
  type Error = SecurityError;
  fn try_from(value: CryptoHandle) -> Result<Self, Self::Error> {
    <Bytes>::from(value).try_into()
  }
}
impl TryFrom<KeyMaterial_AES_GCM_GMAC> for CryptoHandle {
  type Error = SecurityError;
  fn try_from(value: KeyMaterial_AES_GCM_GMAC) -> Result<Self, Self::Error> {
    <Bytes>::try_from(value).map(<CryptoHandle>::from)
  }
}

// Conversions from and into CryptoToken
impl TryFrom<CryptoToken> for KeyMaterial_AES_GCM_GMAC {
  type Error = SecurityError;
  fn try_from(token: CryptoToken) -> Result<Self, Self::Error> {
    BuiltinCryptoToken::try_from(token).map(KeyMaterial_AES_GCM_GMAC::from)
  }
}
impl TryFrom<KeyMaterial_AES_GCM_GMAC> for CryptoToken {
  type Error = SecurityError;
  fn try_from(key_material: KeyMaterial_AES_GCM_GMAC) -> Result<Self, Self::Error> {
    BuiltinCryptoToken::from(key_material).try_into()
  }
}

/// We need to refer to a sequence of key material structures for example in
/// register_local_datawriter.
// Create a wrapper to avoid error E0117
#[allow(non_camel_case_types)] // We use the name from the spec
pub struct KeyMaterial_AES_GCM_GMAC_seq(pub Vec<KeyMaterial_AES_GCM_GMAC>);
// Conversions from and into Bytes for KeyMaterial_AES_GCM_GMAC_seq
impl TryFrom<Bytes> for KeyMaterial_AES_GCM_GMAC_seq {
  type Error = SecurityError;
  fn try_from(value: Bytes) -> Result<Self, Self::Error> {
    // Deserialize CDR-formatted key material
    let serializable_keymat_seq =
      Vec::<Serializable_KeyMaterial_AES_GCM_GMAC>::deserialize(&mut CdrDeserializer::<
        BigEndian, /* TODO: What's the point of this constructor if we need to specify the byte
                    * order anyway */
      >::new_big_endian(
        value.as_ref()
      ))
      .map_err(
        // Map deserialization error to SecurityError
        |e| Self::Error {
          msg: format!("Error deserializing Vec<KeyMaterial_AES_GCM_GMAC>: {}", e),
        },
      )?;

    serializable_keymat_seq
      // Map transformation_kind to builtin for each keymat
      .iter()
      .map(|serializable_keymat| KeyMaterial_AES_GCM_GMAC::try_from(serializable_keymat.clone()))
      // Convert to Vec and dig out the Result
      .collect::<Result<Vec<KeyMaterial_AES_GCM_GMAC>, Self::Error>>()
      // Wrap the Vec
      .map(Self)
  }
}

impl TryFrom<KeyMaterial_AES_GCM_GMAC_seq> for Bytes {
  type Error = SecurityError;
  fn try_from(
    KeyMaterial_AES_GCM_GMAC_seq(keymat_seq): KeyMaterial_AES_GCM_GMAC_seq,
  ) -> Result<Self, Self::Error> {
    // Convert the key material to the serializable structure
    let serializable_keymat_seq = keymat_seq
      .iter()
      .map(|keymat| Serializable_KeyMaterial_AES_GCM_GMAC::from(keymat.clone()))
      .collect();

    // Serialize
    to_bytes::<Vec<Serializable_KeyMaterial_AES_GCM_GMAC>, BigEndian>(&serializable_keymat_seq)
      .map(Bytes::from)
      .map_err(|e| Self::Error {
        msg: format!("Error serializing KeyMaterial_AES_GCM_GMAC_seq: {}", e),
      })
  }
}

// Conversions from and into CryptoHandle for KeyMaterial_AES_GCM_GMAC_seq
impl TryFrom<CryptoHandle> for KeyMaterial_AES_GCM_GMAC_seq {
  type Error = SecurityError;
  fn try_from(value: CryptoHandle) -> Result<Self, Self::Error> {
    <Bytes>::from(value).try_into()
  }
}
impl TryFrom<KeyMaterial_AES_GCM_GMAC_seq> for CryptoHandle {
  type Error = SecurityError;
  fn try_from(value: KeyMaterial_AES_GCM_GMAC_seq) -> Result<Self, Self::Error> {
    <Bytes>::try_from(value).map(<CryptoHandle>::from)
  }
}

// Conversions from and into Vec<CryptoToken> for KeyMaterial_AES_GCM_GMAC_seq
impl TryFrom<Vec<CryptoToken>> for KeyMaterial_AES_GCM_GMAC_seq {
  type Error = SecurityError;
  fn try_from(tokens: Vec<CryptoToken>) -> Result<Self, Self::Error> {
    tokens
      .iter()
      .map(|token| KeyMaterial_AES_GCM_GMAC::try_from(token.clone()))
      .collect::<Result<Vec<KeyMaterial_AES_GCM_GMAC>, Self::Error>>()
      .map(Self)
  }
}
impl TryFrom<KeyMaterial_AES_GCM_GMAC_seq> for Vec<CryptoToken> {
  type Error = SecurityError;
  fn try_from(
    KeyMaterial_AES_GCM_GMAC_seq(keymat_seq): KeyMaterial_AES_GCM_GMAC_seq,
  ) -> Result<Self, Self::Error> {
    keymat_seq
      .iter()
      .map(|keymat| CryptoToken::try_from(keymat.clone()))
      .collect()
  }
}
//For (de)serialization
#[allow(non_camel_case_types)] // We use the name from the spec
#[derive(Deserialize, Serialize, PartialEq, Clone)]
struct Serializable_KeyMaterial_AES_GCM_GMAC {
  transformation_kind: CryptoTransformKind,
  master_salt: Vec<u8>,
  sender_key_id: CryptoTransformKeyId,
  master_sender_key: Vec<u8>,
  receiver_specific_key_id: CryptoTransformKeyId,
  master_receiver_specific_key: Vec<u8>,
}
impl TryFrom<Serializable_KeyMaterial_AES_GCM_GMAC> for KeyMaterial_AES_GCM_GMAC {
  type Error = SecurityError;
  fn try_from(
    Serializable_KeyMaterial_AES_GCM_GMAC {
      transformation_kind,
      master_salt,
      sender_key_id,
      master_sender_key,
      receiver_specific_key_id,
      master_receiver_specific_key,
    }: Serializable_KeyMaterial_AES_GCM_GMAC,
  ) -> Result<Self, Self::Error> {
    // Map transformation_kind to builtin
    BuiltinCryptoTransformationKind::try_from(transformation_kind)
      // Construct a keymat
      .map(|transformation_kind| Self {
        transformation_kind,
        master_salt,
        sender_key_id,
        master_sender_key,
        receiver_specific_key_id,
        master_receiver_specific_key,
      })
  }
}
impl From<KeyMaterial_AES_GCM_GMAC> for Serializable_KeyMaterial_AES_GCM_GMAC {
  fn from(
    KeyMaterial_AES_GCM_GMAC {
      transformation_kind,
      master_salt,
      sender_key_id,
      master_sender_key,
      receiver_specific_key_id,
      master_receiver_specific_key,
    }: KeyMaterial_AES_GCM_GMAC,
  ) -> Self {
    Serializable_KeyMaterial_AES_GCM_GMAC {
      // Serialize transformation_kind
      transformation_kind: transformation_kind.into(),
      master_salt,
      sender_key_id,
      master_sender_key,
      receiver_specific_key_id,
      master_receiver_specific_key,
    }
  }
}

/// Valid values for CryptoTransformKind from section 9.5.2.1.1 of the Security
/// specification (v. 1.1)
#[allow(non_camel_case_types)] // We use the names from the spec
#[derive(Copy, Clone)]
pub enum BuiltinCryptoTransformationKind {
  CRYPTO_TRANSFORMATION_KIND_NONE,
  CRYPTO_TRANSFORMATION_KIND_AES128_GMAC,
  CRYPTO_TRANSFORMATION_KIND_AES128_GCM,
  CRYPTO_TRANSFORMATION_KIND_AES256_GMAC,
  CRYPTO_TRANSFORMATION_KIND_AES256_GCM,
}
impl TryFrom<CryptoTransformKind> for BuiltinCryptoTransformationKind {
  type Error = SecurityError;
  fn try_from(value: CryptoTransformKind) -> Result<Self, Self::Error> {
    match value {
      [0, 0, 0, 0] => Ok(Self::CRYPTO_TRANSFORMATION_KIND_NONE),
      [0, 0, 0, 1] => Ok(Self::CRYPTO_TRANSFORMATION_KIND_AES128_GMAC),
      [0, 0, 0, 2] => Ok(Self::CRYPTO_TRANSFORMATION_KIND_AES128_GCM),
      [0, 0, 0, 3] => Ok(Self::CRYPTO_TRANSFORMATION_KIND_AES256_GMAC),
      [0, 0, 0, 4] => Ok(Self::CRYPTO_TRANSFORMATION_KIND_AES256_GCM),
      _ => Err(Self::Error {
        msg: String::from("Invalid CryptoTransformKind"),
      }),
    }
  }
}
impl From<BuiltinCryptoTransformationKind> for CryptoTransformKind {
  fn from(builtin: BuiltinCryptoTransformationKind) -> CryptoTransformKind {
    match builtin {
      BuiltinCryptoTransformationKind::CRYPTO_TRANSFORMATION_KIND_NONE => [0, 0, 0, 0],
      BuiltinCryptoTransformationKind::CRYPTO_TRANSFORMATION_KIND_AES128_GMAC => [0, 0, 0, 1],
      BuiltinCryptoTransformationKind::CRYPTO_TRANSFORMATION_KIND_AES128_GCM => [0, 0, 0, 2],
      BuiltinCryptoTransformationKind::CRYPTO_TRANSFORMATION_KIND_AES256_GMAC => [0, 0, 0, 3],
      BuiltinCryptoTransformationKind::CRYPTO_TRANSFORMATION_KIND_AES256_GCM => [0, 0, 0, 4],
    }
  }
}

/// CryptoTransformIdentifier type from section 9.5.2.2 of the Security
/// specification (v. 1.1)
pub struct BuiltinCryptoTransformIdentifier {
  pub transformation_kind: BuiltinCryptoTransformationKind,
  pub transformation_key_id: CryptoTransformKeyId,
}
impl TryFrom<CryptoTransformIdentifier> for BuiltinCryptoTransformIdentifier {
  type Error = SecurityError;
  fn try_from(value: CryptoTransformIdentifier) -> Result<Self, Self::Error> {
    match BuiltinCryptoTransformationKind::try_from(value.transformation_kind) {
      Err(e) => Err(e),
      Ok(transformation_kind) => Ok(Self {
        transformation_kind,
        transformation_key_id: value.transformation_key_id,
      }),
    }
  }
}
impl From<BuiltinCryptoTransformIdentifier> for CryptoTransformIdentifier {
  fn from(
    BuiltinCryptoTransformIdentifier {
      transformation_kind,
      transformation_key_id,
    }: BuiltinCryptoTransformIdentifier,
  ) -> Self {
    CryptoTransformIdentifier {
      transformation_kind: transformation_kind.into(),
      transformation_key_id,
    }
  }
}

/// CryptoHeader type from section 9.5.2.3 of the Security specification (v.
/// 1.1)
pub struct BuiltinCryptoHeader {
  pub transform_identifier: BuiltinCryptoTransformIdentifier,
  pub session_id: [u8; 4],
  pub initialization_vector_suffix: [u8; 8],
}
impl TryFrom<CryptoHeader> for BuiltinCryptoHeader {
  type Error = SecurityError;
  fn try_from(
    CryptoHeader {
      transformation_id,
      plugin_crypto_header_extra,
    }: CryptoHeader,
  ) -> Result<Self, Self::Error> {
    let crypto_header_extra = plugin_crypto_header_extra.data;
    //Try to cast [CryptoTransformIdentifier] to [BuiltinCryptoTransformIdentifier]
    // and read 'session_id' and 'initialization_vector_suffix' from
    // 'crypto_header_extra'
    match (
      BuiltinCryptoTransformIdentifier::try_from(transformation_id),
      <[u8; 4]>::try_from(&crypto_header_extra[..4]),
      <[u8; 8]>::try_from(&crypto_header_extra[4..]),
    ) {
      (Ok(transform_identifier), Ok(session_id), Ok(initialization_vector_suffix)) => Ok(Self {
        transform_identifier,
        session_id,
        initialization_vector_suffix,
      }),
      (Err(e), _, _) => Err(e),
      _ => Err(Self::Error {
        msg: format!(
          "plugin_crypto_header_extra was of length {}. Expected 12.",
          crypto_header_extra.len()
        ),
      }),
    }
  }
}
impl From<BuiltinCryptoHeader> for CryptoHeader {
  fn from(
    BuiltinCryptoHeader {
      transform_identifier,
      session_id,
      initialization_vector_suffix,
    }: BuiltinCryptoHeader,
  ) -> Self {
    CryptoHeader {
      transformation_id: transform_identifier.into(),
      plugin_crypto_header_extra: [
        Vec::from(session_id),
        Vec::from(initialization_vector_suffix),
      ]
      .concat()
      .into(),
    }
  }
}

/// CryptoContent type from section 9.5.2.4 of the Security specification (v.
/// 1.1)
pub type BuiltinCryptoContent = CryptoContent;

/// CryptoFooter type from section 9.5.2.5 of the Security specification (v.
/// 1.1)
#[derive(Deserialize, Serialize, PartialEq)]
pub struct BuiltinCryptoFooter {
  pub common_mac: [u8; 16],
  pub receiver_specific_macs: Vec<ReceiverSpecificMAC>,
}
impl TryFrom<Vec<u8>> for BuiltinCryptoFooter {
  type Error = SecurityError;
  fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
    // Deserialize the data
    BuiltinCryptoFooter::deserialize(&mut CdrDeserializer::<
      BigEndian, /* TODO: What's the point of this constructor if we need to specify the byte
                  * order anyway */
    >::new_big_endian(data.as_ref()))
    .map_err(
      // Map deserialization error to SecurityError
      |e| Self::Error {
        msg: format!("Error deserializing BuiltinCryptoFooter: {}", e),
      },
    )
  }
}
impl TryFrom<CryptoFooter> for BuiltinCryptoFooter {
  type Error = SecurityError;
  fn try_from(value: CryptoFooter) -> Result<Self, Self::Error> {
    <Vec<u8>>::from(value).try_into()
  }
}
impl TryFrom<BuiltinCryptoFooter> for Vec<u8> {
  type Error = SecurityError;
  fn try_from(value: BuiltinCryptoFooter) -> Result<Self, Self::Error> {
    // Serialize
    to_bytes::<BuiltinCryptoFooter, BigEndian>(&value).map_err(|e| Self::Error {
      msg: format!("Error serializing BuiltinCryptoFooter: {}", e),
    })
  }
}
impl TryFrom<BuiltinCryptoFooter> for CryptoFooter {
  type Error = SecurityError;
  fn try_from(value: BuiltinCryptoFooter) -> Result<Self, Self::Error> {
    <Vec<u8>>::try_from(value).map(Self::from)
  }
}

/// ReceiverSpecificMAC type from section 9.5.2.5 of the Security specification
/// (v. 1.1)
#[derive(Deserialize, Serialize, PartialEq)]
pub struct ReceiverSpecificMAC {
  pub receiver_mac_key_id: CryptoTransformKeyId,
  pub receiver_mac: [u8; 16],
}
