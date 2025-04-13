pub const QUERY_GET_USER_APP_BY_EMAIL: &str = r#"
SELECT
    id,email,phone_reminder,account_role,is_subscribed,is_enabled,created_at,updated_at
FROM user_app
WHERE email=$1;
"#;

pub const QUERY_GET_USER_PAYM_SUBS: &str = r#"
SELECT
    user_id,
    mp_paym_id,
    payment_idempotency_h,
    transaction_amount,
    installments,
    payment_method_id,
    issuer_id,
    status,
    created_at,
    updated_at
FROM user_sub_payment
WHERE user_id=$1
ORDER BY created_at DESC;
"#;

pub const QUERY_INSERT_NEW_SUB_PAYM: &str = r#"
INSERT INTO user_sub_payment(
    user_id,mp_paym_id,payment_idempotency_h,transaction_amount,
    installments,payment_method_id,issuer_id,status,created_at,updated_at
) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10);
"#;

pub const QUERY_INSERT_PET: &str = r#"
INSERT INTO pet (
    external_id,user_app_id,pet_name,birthday,breed,
    about,is_female,is_lost,is_spaying_neutering,pic,
    created_at,updated_at
) VALUES(
    $1,$2,$3,
    $4,$5,$6,$7,
    $8,$9,
    $10,$11,$12
);
"#;

pub const QUERY_DELETE_PET: &str = r#"DELETE FROM pet WHERE id=$1 AND user_app_id=$2;"#;

pub const QUERY_INSERT_PET_WEIGHT: &str = r#"
INSERT INTO pet_weight (
    pet_id,weight,created_at
) SELECT p.id,$3,$4
FROM pet AS p
WHERE external_id=$1 AND user_app_id=$2
RETURNING id,pet_id,weight AS value,created_at;
"#;

pub const QUERY_INSERT_PET_HEALTH_RECORD: &str = r#"
INSERT INTO pet_health (
    pet_id,health_record,description,created_at
) SELECT p.id,$3,$4,$5
FROM pet AS p
WHERE external_id=$1 AND user_app_id=$2
RETURNING id,pet_id,health_record,description,created_at;
"#;

pub const QUERY_GET_PET_HEALTH_RECORD: &str = r#"
SELECT ph.id,ph.pet_id,ph.health_record,ph.description,ph.created_at
FROM pet_health AS ph
LEFT JOIN pet AS p ON (p.id=ph.pet_id)
WHERE 
    p.external_id = $1 
    AND p.user_app_id = $2
    AND ph.health_record = $3
ORDER BY ph.created_at DESC;
"#;

pub const QUERY_GET_PET_PUBLIC_HEALTH_RECORD: &str = r#"
SELECT ph.id,ph.pet_id,ph.health_record,ph.description,ph.created_at
FROM pet_health AS ph
LEFT JOIN pet AS p ON (p.id=ph.pet_id)
WHERE 
    p.external_id = $1
    AND ph.health_record = $2
ORDER BY ph.created_at DESC;
"#;

pub const QUERY_GET_PET_BY_EXTERNAL_ID: &str = r#"
SELECT
    p.id,p.external_id,pw.weight AS last_weight,p.user_app_id,p.pet_name,
    p.birthday,p.breed,p.about,p.is_female,p.is_lost,
    p.is_spaying_neutering,p.pic,p.created_at,p.updated_at
FROM pet AS p
LEFT JOIN pet_weight pw ON (p.id = pw.pet_id)
WHERE p.external_id = $1
ORDER BY pw.created_at DESC 
LIMIT 1;
"#;

pub const QUERY_GET_PET_BY_ID: &str = r#"
SELECT
    p.id,p.external_id,pw.weight AS last_weight,p.user_app_id,p.pet_name,
    p.birthday,p.breed,p.about,p.is_female,p.is_lost,
    p.is_spaying_neutering,p.pic,p.created_at,p.updated_at
FROM pet AS p
LEFT JOIN pet_weight pw ON (p.id = pw.pet_id)
WHERE p.id = $1 AND p.user_app_id = $2
ORDER BY pw.created_at DESC 
LIMIT 1;
"#;

pub const QUERY_GET_ALL_PETS_USER_ID: &str = r#"
SELECT
    id,external_id,null AS last_weight,user_app_id,pet_name,birthday,breed,
    about,is_female,is_lost,is_spaying_neutering,pic,
    created_at,updated_at
FROM pet
WHERE user_app_id = $1;
"#;

pub const QUERY_UPDATE_PET: &str = r#"
UPDATE pet
    SET pet_name = $3,
    birthday = $4,
    breed = $5,
    about = $6,
    is_female = $7,
    is_lost = $8,
    is_spaying_neutering = $9,
    updated_at = $10
WHERE id = $1 AND user_app_id = $2;
"#;

pub const QUERY_GET_PET_WEIGHTS_BY_EXTERNAL_AND_USER_ID: &str = r#"
SELECT 
    pw.id,pw.pet_id,pw.weight AS value,pw.created_at 
FROM pet_weight AS pw 
LEFT JOIN pet AS p on  (p.id=pw.pet_id)
WHERE 
    p.external_id = $1 AND
    p.user_app_id = $2
ORDER BY pw.created_at DESC;
"#;

pub const QUERY_GET_PET_WEIGHTS_BY_EXTERNAL_ID: &str = r#"
SELECT 
    pw.id,pw.pet_id,pw.weight AS value,pw.created_at 
FROM pet_weight AS pw 
LEFT JOIN pet AS p on  (p.id=pw.pet_id)
WHERE p.external_id = $1
ORDER BY pw.created_at DESC;
"#;

pub const QUERY_DELETE_PET_WEIGHT: &str = r#"
DELETE FROM pet_weight AS pw
WHERE pw.id = $1
AND pw.pet_id IN (
    SELECT p.id
    FROM pet AS p
    WHERE 
        p.external_id = $2 AND
        p.user_app_id = $3
    LIMIT 1
);
"#;

pub const QUERY_DELETE_PET_HEALTH_RECORD: &str = r#"
DELETE FROM pet_health AS ph
WHERE 
    ph.id = $1
    AND ph.health_record = $2
    AND ph.pet_id IN (
        SELECT p.id
        FROM pet AS p
        WHERE 
            p.external_id = $3 AND
            p.user_app_id = $4
        LIMIT 1
    );
"#;

pub const QUERY_GET_OWNER_CONTACTS: &str = r#"
SELECT 
    id,user_app_id,full_name,contact_value,created_at
FROM owner_contact
WHERE user_app_id=$1
ORDER BY created_at DESC;
"#;

pub const QUERY_GET_PET_OWNER_CONTACTS: &str = r#"
SELECT 
    c.id,c.user_app_id,c.full_name,c.contact_value,c.created_at
FROM owner_contact AS c
LEFT JOIN pet AS p ON (p.user_app_id = c.user_app_id)
WHERE p.external_id=$1
ORDER BY c.created_at DESC;
"#;

pub const QUERY_INSERT_NEW_OWNER_CONTACT: &str = r#"
INSERT INTO owner_contact(
    user_app_id,full_name,contact_value,created_at
) VALUES (
    $1,$2,$3,$4
);
"#;

pub const QUERY_DELETE_OWNER_CONTACT: &str = r#"
DELETE FROM owner_contact
WHERE 
    id = $1
    AND user_app_id = $2;
"#;

pub const QUERY_INSERT_PET_NOTE: &str = r#"
INSERT INTO pet_note (
    pet_id,title,content,created_at
) SELECT p.id,$3,$4,$5
FROM pet AS p
WHERE
    p.id = $1 AND
    p.user_app_id=$2;
"#;

pub const QUERY_GET_PET_NOTES: &str = r#"
SELECT 
    pn.id, pn.pet_id, pn.title, pn.content, pn.created_at, pn.updated_at
FROM pet_note AS pn
LEFT JOIN pet AS p ON (p.id = pn.pet_id)
WHERE p.id=$1 AND p.user_app_id=$2;
"#;

pub const QUERY_DELETE_PET_NOTE: &str = r#"
DELETE FROM pet_note AS pn
WHERE 
    pn.id = $1
    AND pn.pet_id IN (
        SELECT p.id
        FROM pet AS p
        WHERE 
            p.id = $2 AND
            p.user_app_id = $3
        LIMIT 1
    );
"#;

pub const QUERY_INSERT_USER_REMINDER: &str = r#"
INSERT INTO reminder(
    user_app_id,body,execution_id,notification_type,send_at,user_timezone,created_at
) VALUES($1,$2,$3,$4,$5,$6,$7);
"#;

pub const QUERY_GET_USER_ACTIVE_REMINDERS: &str = r#"
SELECT 
    r.id,r.user_app_id,r.body,r.execution_id,
    r.notification_type,r.send_at,r.user_timezone,
    r.created_at
FROM reminder AS r
WHERE r.user_app_id = $1 AND r.send_at>=$2
"#;

// TODO: Stop reminder executions
pub const QUERY_DELETE_USER_APP_DATA: &str = r#"
DELETE FROM pet WHERE user_app_id = $1;
DELETE FROM owner_contact WHERE user_app_id = $1;
DELETE FROM reminder WHERE user_app_id = $1;
DELETE FROM user_sub_payment WHERE user_id = $1;
UPDATE user_app SET is_enabled=0,is_subscribed=0,phone_reminder=NULL,updated_at=$2 WHERE id = $1;
"#;
