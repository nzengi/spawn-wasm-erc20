use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// ERC-20 Token Standardı
#[wasm_bindgen]
pub struct ERC20 {
    name: String,
    symbol: String,
    total_supply: u64,  // WASM ile uyumlu hale getirmek için u128 yerine u64 kullanıyoruz
    max_supply: u64,    // Maksimum arz belirlenmiştir.
    balances: HashMap<String, u64>,  // Kullanıcı bakiyeleri
    allowances: HashMap<String, HashMap<String, u64>>,  // İzinler (owner -> spender -> miktar)
}

#[wasm_bindgen]
impl ERC20 {
    /// Yeni bir ERC-20 token oluşturur.
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str, symbol: &str, initial_supply: u64, max_supply: u64) -> ERC20 {
        let mut balances = HashMap::new();
        balances.insert("owner".to_string(), initial_supply); // Sahip başlangıç tokenlarına sahip olur

        ERC20 {
            name: name.to_string(),
            symbol: symbol.to_string(),
            total_supply: initial_supply,
            max_supply,
            balances,
            allowances: HashMap::new(),
        }
    }

    /// Token adını döndürür.
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Token sembolünü döndürür.
    pub fn symbol(&self) -> String {
        self.symbol.clone()
    }

    /// Toplam arzı döndürür.
    pub fn total_supply(&self) -> u64 {
        self.total_supply
    }

    /// Maksimum arzı döndürür.
    pub fn max_supply(&self) -> u64 {
        self.max_supply
    }

    /// Bir kullanıcının bakiyesini sorgular.
    pub fn balance_of(&self, owner: &str) -> u64 {
        *self.balances.get(owner).unwrap_or(&0)
    }

    /// Token transfer eder. `from` hesabından `to` hesabına `amount` kadar token transfer eder.
    /// Re-entrancy saldırılarını önlemek için izin ve transfer mantığı ayrı tutulur.
    pub fn transfer(&mut self, from: &str, to: &str, amount: u64) -> bool {
        if from == to || amount == 0 {
            return false;  // Kendine transfer veya sıfır transfer geçersizdir.
        }

        let sender_balance = self.balances.get(from).unwrap_or(&0);
        if *sender_balance < amount {
            return false;  // Yetersiz bakiye
        }

        // Önce transfer yapılır, ardından onay ve izin güncellenir
        self.balances.insert(from.to_string(), sender_balance - amount);
        let recipient_balance = self.balances.get(to).unwrap_or(&0);
        self.balances.insert(to.to_string(), recipient_balance + amount);

        // Olayı logla
        self.log_event("Transfer", &format!("From: {}, To: {}, Amount: {}", from, to, amount));
        true
    }

    /// Bir kullanıcıya belirli miktarda token harcaması için izin verir.
    pub fn approve(&mut self, owner: &str, spender: &str, amount: u64) -> bool {
        if owner == spender {
            return false; // Kullanıcının kendine harcama izni vermesi engellenir
        }

        let owner_allowances = self.allowances.entry(owner.to_string()).or_insert(HashMap::new());
        owner_allowances.insert(spender.to_string(), amount);

        // Olayı logla
        self.log_event("Approval", &format!("Owner: {}, Spender: {}, Amount: {}", owner, spender, amount));
        true
    }

    /// Harcama iznini sorgular.
    pub fn allowance(&self, owner: &str, spender: &str) -> u64 {
        self.allowances
            .get(owner)
            .and_then(|allowances| allowances.get(spender))
            .cloned()
            .unwrap_or(0)
    }

    /// Harcama izni ile transfer işlemi gerçekleştirir.
    pub fn transfer_from(&mut self, owner: &str, spender: &str, to: &str, amount: u64) -> bool {
        let allowance = self.allowance(owner, spender);
        if allowance < amount {
            return false;  // Yetersiz izin miktarı
        }

        // Önce izin güncellenir, sonra transfer yapılır.
        if self.transfer(owner, to, amount) {
            self.approve(owner, spender, allowance - amount);  // İzin miktarını güncelle
            return true;
        }
        false
    }

    /// Token yakan (burn) fonksiyonu. Toplam arzı azaltır.
    pub fn burn(&mut self, owner: &str, amount: u64) -> bool {
        let owner_balance = self.balances.get(owner).unwrap_or(&0);
        if *owner_balance < amount {
            return false;  // Yetersiz bakiye
        }

        self.balances.insert(owner.to_string(), owner_balance - amount);
        self.total_supply -= amount;

        // Olayı logla
        self.log_event("Burn", &format!("Owner: {}, Amount: {}", owner, amount));
        true
    }

    /// Yeni token basan (mint) fonksiyonu. Toplam arzı artırır.
    /// Maksimum arz kontrolü yapılır.
    pub fn mint(&mut self, recipient: &str, amount: u64) -> bool {
        if self.total_supply + amount > self.max_supply {
            return false;  // Maksimum arz aşılamaz.
        }

        let recipient_balance = self.balances.get(recipient).unwrap_or(&0);
        self.balances.insert(recipient.to_string(), recipient_balance + amount);
        self.total_supply += amount;

        // Olayı logla
        self.log_event("Mint", &format!("Recipient: {}, Amount: {}", recipient, amount));
        true
    }

    /// Olayları loglamak için basit bir log fonksiyonu
    fn log_event(&self, event_name: &str, details: &str) {
        web_sys::console::log_2(&event_name.into(), &details.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_supply() {
        let token = ERC20::new("Spawn Token", "SPN", 1000, 2000);
        assert_eq!(token.balance_of("owner"), 1000);
    }

    #[test]
    fn test_transfer() {
        let mut token = ERC20::new("Spawn Token", "SPN", 1000, 2000);
        let success = token.transfer("owner", "user1", 200);
        assert!(success);
        assert_eq!(token.balance_of("user1"), 200);
        assert_eq!(token.balance_of("owner"), 800);
    }

    #[test]
    fn test_approve_and_transfer_from() {
        let mut token = ERC20::new("Spawn Token", "SPN", 1000, 2000);
        token.approve("owner", "user1", 100);
        let success = token.transfer_from("owner", "user1", "user2", 50);
        assert!(success);
        assert_eq!(token.balance_of("user2"), 50);
        assert_eq!(token.allowance("owner", "user1"), 50);
    }

    #[test]
    fn test_burn() {
        let mut token = ERC20::new("Spawn Token", "SPN", 1000, 2000);
        let success = token.burn("owner", 100);
        assert!(success);
        assert_eq!(token.total_supply(), 900);
        assert_eq!(token.balance_of("owner"), 900);
    }

    #[test]
    fn test_mint() {
        let mut token = ERC20::new("Spawn Token", "SPN", 1000, 2000);
        let success = token.mint("user1", 200);
        assert!(success);
        assert_eq!(token.total_supply(), 1200);
        assert_eq!(token.balance_of("user1"), 200);
    }

    #[test]
    fn test_max_supply_limit() {
        let mut token = ERC20::new("Spawn Token", "SPN", 1000, 1500);
        let success = token.mint("user1", 600);
        assert!(!success);  // Maksimum arz aşılmamalı
        assert_eq!(token.total_supply(), 1000);
    }
}
