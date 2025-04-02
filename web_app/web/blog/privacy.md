# Política de privacidad

Esta [política de privacidad](https://www.freeprivacypolicy.com/live/7c4516b2-ace1-4758-bfc3-9873e937468c) detalla como se almacenan y el uso que se da a los datos ingresados por los usuarios.

Pet-Info utiliza el [servicio de Google Oauth](https://support.google.com/a/answer/11609141?sjid=14674853996002502171-NC) para dar de alta un usuario e iniciar sesión. Por lo que, no se almacenan contraseñas.

Aws es el provedor de toda la infraestructura de toda la aplicación. Por ejemplo:
  - La aplicación web esta desplegada en [ec2 instance de aws](https://aws.amazon.com/es/ec2/instance-types/)
  - las notificaciones se agendan usando [step-functions](https://aws.amazon.com/es/step-functions/) y [lambdas](https://aws.amazon.com/es/lambda/)
  - almacenamiento en general: [s3](https://aws.amazon.com/es/s3/)

## Pagos
Pet-Info usa el [api de Mercado Pago](https://www.mercadopago.com.mx/developers/es/docs)

## Mensajes
Pet-Info usa la [api proporcionada por meta para WhatsApp Business](https://developers.facebook.com/docs/whatsapp/?locale=es_LA)

## Información personal que almacenamos

### Informacion personal
- correo electronico
- numero(s) de telofono(s)
- nombres

### Informacion de terceros
- Google: para valdiar el correo y asociar la cuenta a dicho correo


## Como almacenamos y protegemos tu información
Toda la informacion es encriptada en los servidores y cuando se hacen peticiones al servidor
