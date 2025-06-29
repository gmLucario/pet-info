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
WHERE 
    user_id=$1 AND
    (status=$2 OR $2='all')
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
    user_app_id,pet_name,birthday,breed,
    about,is_female,is_lost,is_spaying_neutering,pic,
    created_at,updated_at
) VALUES(
    $1,$2,$3,
    $4,$5,$6,$7,
    $8,$9,
    $10,$11
);
"#;

pub const QUERY_INSERT_PET_EXTERNAL_ID: &str = r#"
INSERT INTO pet_external_id(external_id,created_at) VALUES ($1,$2);
"#;

pub const QUERY_LINK_PET_WITH_EXTERNAL_ID: &str = r#"
INSERT INTO pet_linked (pet_id,id_pet_external_id) VALUES($1,$2);
"#;

pub const QUERY_IS_PET_EXTERNAL_ID_LINKED: &str = r#"
SELECT 
	CASE WHEN pl.pet_id IS NOT NULL THEN 1 ELSE 0 End AS is_linked
FROM pet_external_id pei 
LEFT JOIN pet_linked pl ON (pl.id_pet_external_id = pei.id ) 
WHERE pei.external_id = $1;
"#;

pub const QUERY_DELETE_PET: &str = r#"DELETE FROM pet WHERE id=$1 AND user_app_id=$2;"#;

pub const QUERY_INSERT_PET_WEIGHT: &str = r#"
INSERT INTO pet_weight (
    pet_id,weight,created_at
) SELECT p.id,$3,$4
FROM pet AS p
LEFT JOIN pet_linked AS pidlink ON (p.id=pidlink.pet_id)
LEFT JOIN pet_external_id AS peid ON (peid.id=pidlink.id_pet_external_id)
WHERE peid.external_id=$1 AND user_app_id=$2
RETURNING id,pet_id,weight AS value,created_at;
"#;

pub const QUERY_INSERT_PET_HEALTH_RECORD: &str = r#"
INSERT INTO pet_health (
    pet_id,health_record,description,created_at
) SELECT p.id,$3,$4,$5
FROM pet AS p
LEFT JOIN pet_linked AS pidlink ON (p.id=pidlink.pet_id)
LEFT JOIN pet_external_id AS peid ON (peid.id=pidlink.id_pet_external_id)
WHERE peid.external_id=$1 AND user_app_id=$2
RETURNING id,pet_id,health_record,description,created_at;
"#;

pub const QUERY_GET_PET_HEALTH_RECORD: &str = r#"
SELECT ph.id,ph.pet_id,ph.health_record,ph.description,ph.created_at
FROM pet_external_id AS peid
INNER JOIN pet_linked AS pidlink ON (peid.id = pidlink.id_pet_external_id)
INNER JOIN pet AS p ON (p.id = pidlink.pet_id)
INNER JOIN pet_health AS ph ON (p.id = ph.pet_id)
WHERE 
    peid.external_id = $1 
    AND p.user_app_id = $2
    AND ph.health_record = $3
ORDER BY ph.created_at DESC;
"#;

pub const QUERY_GET_PET_PUBLIC_HEALTH_RECORD: &str = r#"
SELECT ph.id,ph.pet_id,ph.health_record,ph.description,ph.created_at
FROM pet_external_id AS peid
INNER JOIN pet_linked AS pidlink ON (peid.id = pidlink.id_pet_external_id)
INNER JOIN pet AS p ON (p.id = pidlink.pet_id)
INNER JOIN pet_health AS ph ON (p.id = ph.pet_id)
WHERE 
    peid.external_id = $1
    AND ph.health_record = $2
ORDER BY ph.created_at DESC;
"#;

pub const QUERY_GET_PET_BY_EXTERNAL_ID: &str = r#"
SELECT
    p.id,peid.external_id,pw.weight AS last_weight,p.user_app_id,p.pet_name,
    p.birthday,p.breed,p.about,p.is_female,p.is_lost,
    p.is_spaying_neutering,p.pic,p.created_at,p.updated_at
FROM pet AS p
LEFT JOIN pet_linked AS pidlink ON (p.id=pidlink.pet_id)
LEFT JOIN pet_external_id AS peid ON (peid.id=pidlink.id_pet_external_id)
LEFT JOIN pet_weight pw ON (p.id = pw.pet_id)
WHERE peid.external_id = $1
ORDER BY pw.created_at DESC 
LIMIT 1;
"#;

pub const QUERY_GET_PET_BY_ID: &str = r#"
SELECT
    p.id,peid.external_id,pw.weight AS last_weight,p.user_app_id,p.pet_name,
    p.birthday,p.breed,p.about,p.is_female,p.is_lost,
    p.is_spaying_neutering,p.pic,p.created_at,p.updated_at
FROM pet AS p
LEFT JOIN pet_linked AS pidlink ON (p.id=pidlink.pet_id)
LEFT JOIN pet_external_id AS peid ON (peid.id=pidlink.id_pet_external_id)
LEFT JOIN pet_weight pw ON (p.id = pw.pet_id)
WHERE p.id = $1 AND p.user_app_id = $2
ORDER BY pw.created_at DESC 
LIMIT 1;
"#;

pub const QUERY_GET_ALL_PETS_USER_ID: &str = r#"
SELECT
    pet.id,peid.external_id,pw.weight AS last_weight,user_app_id,pet_name,birthday,breed,
    about,is_female,is_lost,is_spaying_neutering,pic,
    pet.created_at,pet.updated_at
FROM pet
LEFT JOIN pet_linked AS pl ON (pl.pet_id=pet.id)
LEFT JOIN pet_external_id AS peid ON (peid.id=pl.id_pet_external_id)
LEFT JOIN (
    SELECT pet_id, weight, 
           ROW_NUMBER() OVER (PARTITION BY pet_id ORDER BY created_at DESC) as rn
    FROM pet_weight
) pw ON (pw.pet_id = pet.id AND pw.rn = 1)
WHERE user_app_id = $1
ORDER BY pet.created_at DESC;
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
FROM pet_external_id AS peid
INNER JOIN pet_linked AS plinked ON (peid.id = plinked.id_pet_external_id)
INNER JOIN pet AS p ON (p.id = plinked.pet_id)
INNER JOIN pet_weight AS pw ON (p.id = pw.pet_id)
WHERE 
    peid.external_id = $1 AND
    p.user_app_id = $2
ORDER BY pw.created_at DESC;
"#;

pub const QUERY_GET_PET_PUBLIC_PIC_BY_EXTERNAL_ID: &str = r#"
SELECT p.pic
FROM pet AS p
LEFT JOIN pet_linked AS plinked ON (p.id=plinked.pet_id)
LEFT JOIN pet_external_id AS peid ON (peid.id=plinked.id_pet_external_id)
WHERE peid.external_id = $1;
"#;

pub const QUERY_GET_PET_WEIGHTS_BY_EXTERNAL_ID: &str = r#"
SELECT 
    pw.id,pw.pet_id,pw.weight AS value,pw.created_at 
FROM pet_external_id AS peid
INNER JOIN pet_linked AS plinked ON (peid.id = plinked.id_pet_external_id)
INNER JOIN pet AS p ON (p.id = plinked.pet_id)
INNER JOIN pet_weight AS pw ON (p.id = pw.pet_id)
WHERE peid.external_id = $1
ORDER BY pw.created_at DESC;
"#;

pub const QUERY_DELETE_PET_WEIGHT: &str = r#"
DELETE FROM pet_weight 
WHERE id = $1
AND pet_id = (
    SELECT p.id
    FROM pet_external_id AS peid
    INNER JOIN pet_linked AS plinked ON (peid.id = plinked.id_pet_external_id)
    INNER JOIN pet AS p ON (p.id = plinked.pet_id)
    WHERE 
        peid.external_id = $2 AND
        p.user_app_id = $3
    LIMIT 1
);
"#;

pub const QUERY_DELETE_PET_HEALTH_RECORD: &str = r#"
DELETE FROM pet_health 
WHERE 
    id = $1
    AND health_record = $2
    AND pet_id = (
        SELECT p.id
        FROM pet_external_id AS peid
        INNER JOIN pet_linked AS plinked ON (peid.id = plinked.id_pet_external_id)
        INNER JOIN pet AS p ON (p.id = plinked.pet_id)
        WHERE 
            peid.external_id = $3 AND
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
LEFT JOIN pet_linked AS plinked ON (p.id=plinked.pet_id)
LEFT JOIN pet_external_id AS peid ON (peid.id=plinked.id_pet_external_id)
WHERE peid.external_id=$1
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

pub const QUERY_DELETE_USER_APP_DATA: &str = r#"
DELETE FROM pet_external_id AS pexid WHERE pexid.id IN (
    SELECT plink.id_pet_external_id FROM pet_linked AS plink
    LEFT JOIN pet AS p on (p.id=plink.pet_id)
    WHERE p.user_app_id = $1
);
DELETE FROM pet WHERE user_app_id = $1;
DELETE FROM owner_contact WHERE user_app_id = $1;
DELETE FROM reminder WHERE user_app_id = $1;
DELETE FROM user_sub_payment WHERE user_id = $1;
UPDATE user_app SET is_enabled=0,is_subscribed=0,phone_reminder=NULL,updated_at=$2 WHERE id = $1;
"#;
