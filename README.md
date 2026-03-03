🚗 SOL Car P2P Ontario
Compraventa de vehículos usados con retención automática de impuesto HST en Solana

🇨🇦 El problema real
En Ontario, Canadá, la compraventa de un vehículo usado entre particulares (P2P) funciona así:

Vendedor y comprador se ponen de acuerdo en el precio
El vendedor firma la tarjeta de registro del vehículo
El comprador debe ir físicamente a Service Ontario a pagar el HST (13%) y tramitar las placas
Muchos compradores nunca van — el gobierno no cobra el impuesto y el cambio de dueño queda en limbo
No hay forma instantánea de verificar si un auto tiene safety certificate, reportes de robo o historial de dueños sin pagar servicios como Carfax

El resultado: evasión fiscal, fraude en ventas, y burocracia innecesaria.

💡 La solución
Este proyecto imagina un escenario donde el Ministry of Transportation of Ontario (MTO) ha tokenizado todos los vehículos registrados como NFTs en la blockchain de Solana.
Cuando dos particulares realizan una compraventa:
Vendedor y comprador están frente al auto
            ↓
Ambos conectan su wallet
            ↓
Vendedor lista el vehículo con el precio acordado
            ↓
Comprador ejecuta la transacción
            ↓
El contrato divide automáticamente:
  ├── Precio acordado  →  Vendedor (al instante)
  ├── 13% HST          →  MTO Ontario (automático)
  └── 0.5% fee         →  Protocolo
            ↓
El NFT del vehículo se transfiere al comprador
Todo en una sola transacción. Sin filas. Sin papel.

✨ Características

✅ Registro de vehículos por MTO — solo el gobierno puede crear activos
✅ Safety certificate on-chain — verificable públicamente sin pagar Carfax
✅ Reporte de robo — MTO puede marcar/desmarcar vehículos robados
✅ Bloqueo automático — no se puede vender un auto robado
✅ HST 13% automático — el impuesto llega al gobierno en la misma transacción
✅ Historial de dueños — cada transferencia queda registrada on-chain
✅ Cancelación de venta — el vendedor puede retirar el listing en cualquier momento


🏗️ Arquitectura del contrato
Cuentas (PDAs)
ProgramState — Estado global del protocolo
authority          → wallet del MTO Ontario
government_wallet  → wallet que recibe el HST
tax_basis_points   → 1300 (13% HST)
fee_basis_points   → configurable
total_transactions → contador global
VehicleRecord — Un registro por cada vehículo
vin              → Vehicle Identification Number (17 chars)
make / model     → marca y modelo
year / color     → año y color
owner            → dueño actual (Pubkey)
has_safety       → safety certificate vigente
is_stolen        → reporte de robo activo
is_listed        → está en venta
asking_price     → precio en lamports
transfer_count   → historial de transferencias
last_sale_price  → último precio de venta
Instrucciones
InstrucciónQuiénDescripcióninitializeMTO OntarioConfigura el protocolo una sola vezregister_vehicleMTO OntarioTokeniza un vehículo como NFTupdate_safetyMTO OntarioActualiza el safety certificateflag_stolenMTO OntarioMarca o desmarca robolist_for_saleDueñoPone el vehículo en venta con precioexecute_saleCompradorEjecuta la compra y distribuye fondoscancel_listingDueñoRetira el vehículo de venta

🧪 Tests
El proyecto incluye 5 tests que validan el flujo completo:
✅ MTO Ontario inicializa el sistema
✅ MTO Ontario registra un vehículo (Toyota Tacoma 2022)
✅ No se puede vender un auto robado
✅ Vendedor lista el auto para venta
✅ Comprador ejecuta la compra y HST va al gobierno automáticamente
Para correr los tests:
bashanchor test --skip-deploy
Resultado esperado:
5 passing (2s)

🚀 Deploy
Program ID en Devnet:
EQDnpsPH5W2TBATpoYdo7c3a5DKq7X12co14MVn7gB8B
Verificar en Solana Explorer:
https://explorer.solana.com/address/EQDnpsPH5W2TBATpoYdo7c3a5DKq7X12co14MVn7gB8B?cluster=devnet

🛠️ Instalación y uso
Requisitos

Rust
Solana CLI
Anchor CLI 0.32+
Node.js 20+
Yarn

Clonar y compilar
bashgit clone https://github.com/MemoLabPRO/sol-car-p2p-ontario
cd sol-car-p2p-ontario
yarn install
anchor build
Configurar red
bashsolana config set --url devnet
Correr tests
bashanchor test --skip-deploy

🌎 Impacto potencial
Este modelo es aplicable a cualquier jurisdicción donde:

Exista un registro gubernamental de vehículos
Se cobre un impuesto por compraventa
Se quiera eliminar la evasión fiscal y el fraude

Países con alta aplicabilidad: Canadá, México, Colombia, Guatemala — cualquier lugar donde la informalidad en compraventas de vehículos sea un problema sistémico.

👨‍💻 Autor
Desarrollado por MemoLabPRO como parte del programa Solana Developer Bootcamp organizado por la Solana Foundation.

📄 Licencia
MIT