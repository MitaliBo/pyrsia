/*
   Copyright 2021 JFrog Ltd

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/
use signed::signed::{JwsSignatureAlgorithms, Signed};
use signed_struct::signed_struct;

#[signed_struct]
struct Identity {
    public_key: Vec<u8>,
    identity_algorithm: JwsSignatureAlgorithms,
    name: String,
    description: Option<String>,
    email: Option<String>,
    web_url: Option<String>,
    phone_number: Option<String>,
}
